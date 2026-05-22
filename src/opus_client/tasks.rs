use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use chrono::{DateTime, Utc};
use crate::debug::debug_log;
use crate::opus::models::{Task, Column};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpusTaskCreate {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpusTaskUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_id: Option<String>,
}


impl super::OpusClient {
    pub async fn create_task_with_magic(
        &self,
        magic_text: &str,
        default_project_id: &str,
    ) -> Result<Task, Box<dyn Error + Send + Sync>> {
        debug_log(&format!("Parsing magic text: '{}'", magic_text));
        let parsed = self.parser.parse(magic_text);
        debug_log(&format!(
            "Parsed task - title: '{}', labels: {:?}, project: {:?}",
            parsed.title, parsed.labels, parsed.project
        ));

        let project_id = if let Some(project_name) = &parsed.project {
            debug_log(&format!("Looking up project: '{}'.", project_name));
            match self.find_or_get_project_id(project_name).await {
                Ok(Some(id)) => {
                    debug_log(&format!("Found project ID: {} for project '{}'.", id, project_name));
                    id
                }
                Ok(None) => {
                    debug_log(&format!(
                        "Project '{}' not found, using default: {}.",
                        project_name, default_project_id
                    ));
                    default_project_id.to_string()
                }
                Err(e) => {
                    debug_log(&format!(
                        "Error looking up project '{}': {}. Using default: {}.",
                        project_name, e, default_project_id
                    ));
                    default_project_id.to_string()
                }
            }
        } else {
            default_project_id.to_string()
        };

        let priority_str = parsed.priority.clone();

        let create_payload = OpusTaskCreate {
            title: parsed.title.clone(),
            description: None,
            priority: priority_str,
            due_date: parsed.due_date,
            start_date: parsed.start_date,
            column_id: None,
        };

        debug_log(&format!(
            "Creating task with project_id: {}, title: '{}'",
            project_id, create_payload.title
        ));
        let created_task = self.create_task(&project_id, &create_payload).await?;
        debug_log(&format!("Task created with ID: {}", created_task.id));

        for label_name in &parsed.labels {
            debug_log(&format!("Processing label: '{}'", label_name));
            match self.ensure_label_exists(label_name).await {
                Ok(label) => {
                    debug_log(&format!("Label '{}' exists/created with ID: {}", label_name, label.id));
                    match self.attach_label(&label.id, &created_task.id).await {
                        Ok(_) => debug_log(&format!(
                            "Successfully added label '{}' to task {}",
                            label_name, created_task.id
                        )),
                        Err(e) => debug_log(&format!(
                            "Failed to add label '{}' to task {}: {}",
                            label_name, created_task.id, e
                        )),
                    }
                }
                Err(e) => debug_log(&format!(
                    "Failed to ensure label '{}' exists: {}",
                    label_name, e
                )),
            }
        }

        for username in &parsed.assignees {
            if let Some(user) = self.find_user_by_name(username).await {
                let _ = self.update_task_assignee(&created_task.id, &user.id).await;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        debug_log(&format!(
            "SUCCESS: Task created successfully! ID: {}, Title: '{}'",
            created_task.id, created_task.title
        ));

        Ok(created_task)
    }

    pub async fn create_task(
        &self,
        project_id: &str,
        task: &OpusTaskCreate,
    ) -> Result<Task, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/task/{}", self.base_url, project_id);
        debug_log(&format!("Making POST request to: {}", url));

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(task)
            .send()
            .await?;

        let status = response.status();
        debug_log(&format!("Response status: {}", status));
        if status.is_success() {
            let created: Task = response.json().await?;
            debug_log(&format!("Successfully created task: {}", created.id));
            Ok(created)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());
            debug_log(&format!("create_task API error ({}): {}", status, error_text));
            Err(format!("Failed to create task: {} - {}", status, error_text).into())
        }
    }

    pub async fn get_task(&self, task_id: &str) -> Result<Task, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/task/{}", self.base_url, task_id);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to get task {}: {} - {}", task_id, status, error_text).into())
        }
    }

    pub async fn update_task(
        &self,
        task_id: &str,
        update: &OpusTaskUpdate,
    ) -> Result<Task, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/task/{}", self.base_url, task_id);
        debug_log(&format!("Making PUT request to: {}", url));

        let json_str = serde_json::to_string(update).unwrap_or_default();
        debug_log(&format!("update_task JSON payload: {}", json_str));

        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(update)
            .send()
            .await?;

        let status = response.status();
        debug_log(&format!("Response status: {}", status));
        if status.is_success() {
            let updated: Task = response.json().await?;
            debug_log(&format!("Successfully updated task: {}", updated.id));
            Ok(updated)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            debug_log(&format!("update_task API error ({}): {}", status, error_text));
            Err(format!("Failed to update task: {} - {}", status, error_text).into())
        }
    }

    pub async fn delete_task(&self, task_id: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/task/{}", self.base_url, task_id);
        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to delete task {}: {} - {}", task_id, status, error_text).into())
        }
    }

    pub async fn update_task_status(
        &self,
        task_id: &str,
        status: &str,
        column_id: &str,
    ) -> Result<Task, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/task/status/{}", self.base_url, task_id);
        let payload = serde_json::json!({
            "status": status,
            "columnId": column_id,
        });
        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&payload)
            .send()
            .await?;

        let resp_status = response.status();
        if resp_status.is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to update task status: {} - {}", resp_status, error_text).into())
        }
    }

    pub async fn update_task_priority(
        &self,
        task_id: &str,
        priority: &str,
    ) -> Result<Task, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/task/priority/{}", self.base_url, task_id);
        let payload = serde_json::json!({ "priority": priority });
        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to update priority: {} - {}", status, error_text).into())
        }
    }

    pub async fn update_task_assignee(
        &self,
        task_id: &str,
        user_id: &str,
    ) -> Result<Task, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/task/assignee/{}", self.base_url, task_id);
        let payload = serde_json::json!({ "userId": user_id });
        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to update assignee: {} - {}", status, error_text).into())
        }
    }

    pub async fn update_task_due_date(
        &self,
        task_id: &str,
        due_date: &str,
    ) -> Result<Task, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/task/due-date/{}", self.base_url, task_id);
        let payload = serde_json::json!({ "dueDate": due_date });
        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to update due date: {} - {}", status, error_text).into())
        }
    }

    pub async fn update_task_title(
        &self,
        task_id: &str,
        title: &str,
    ) -> Result<Task, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/task/title/{}", self.base_url, task_id);
        let payload = serde_json::json!({ "title": title });
        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to update title: {} - {}", status, error_text).into())
        }
    }

    pub async fn toggle_task_status(
        &self,
        task_id: &str,
        current_status: &str,
        columns: &[Column],
    ) -> Result<Task, Box<dyn Error + Send + Sync>> {
        let is_final = columns
            .iter()
            .any(|c| c.slug == current_status && c.is_final);

        let target_column = if is_final {
            columns.iter().min_by_key(|c| c.position)
        } else {
            columns.iter().find(|c| c.is_final)
        };

        match target_column {
            Some(col) => self.update_task_status(task_id, &col.slug, &col.id).await,
            None => Err("No suitable column found for status toggle".into()),
        }
    }

    pub async fn get_tasks_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<Task>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/task/tasks/{}", self.base_url, project_id);
        debug_log(&format!("Fetching tasks for project {}: {}", project_id, url));

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let tasks: Vec<Task> = response.json().await?;
            debug_log(&format!(
                "Got {} tasks for project {}",
                tasks.len(),
                project_id
            ));
            Ok(tasks)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            debug_log(&format!(
                "Failed to fetch tasks for project {}: {} - {}",
                project_id, status, error_text
            ));
            Err(format!(
                "Failed to fetch tasks for project {}: {} - {}",
                project_id, status, error_text
            )
            .into())
        }
    }

    pub async fn get_all_tasks(&self) -> Result<Vec<Task>, Box<dyn Error + Send + Sync>> {
        debug_log("Starting comprehensive task fetch across all projects...");

        let projects = self.get_all_projects().await?;
        let mut all_tasks = Vec::new();

        for project in &projects {
            match self.get_tasks_for_project(&project.id).await {
                Ok(mut project_tasks) => {
                    debug_log(&format!(
                        "Project '{}' returned {} tasks",
                        project.name,
                        project_tasks.len()
                    ));
                    all_tasks.append(&mut project_tasks);
                }
                Err(e) => {
                    debug_log(&format!(
                        "Failed to fetch tasks from project '{}': {}",
                        project.name, e
                    ));
                    continue;
                }
            }
        }

        debug_log(&format!(
            "Comprehensive fetch complete: {} total tasks",
            all_tasks.len()
        ));
        Ok(all_tasks)
    }

    pub async fn get_tasks_with_projects(
        &self,
    ) -> Result<
        (
            Vec<Task>,
            HashMap<String, String>,
            HashMap<String, String>,
        ),
        Box<dyn Error + Send + Sync>,
    > {
        let projects = self.get_all_projects().await?;

        let mut project_map = HashMap::new();
        let mut project_colors = HashMap::new();
        for project in &projects {
            project_map.insert(project.id.clone(), project.name.clone());
            let color = project
                .icon
                .clone()
                .unwrap_or_default();
            project_colors.insert(project.id.clone(), color);
        }

        let mut all_tasks = Vec::new();
        for project in &projects {
            match self.get_tasks_for_project(&project.id).await {
                Ok(mut tasks) => {
                    all_tasks.append(&mut tasks);
                }
                Err(e) => {
                    debug_log(&format!(
                        "Failed to fetch tasks from project '{}': {}",
                        project.name, e
                    ));
                }
            }
        }

        debug_log(&format!(
            "get_tasks_with_projects: {} tasks, {} projects",
            all_tasks.len(),
            project_map.len()
        ));

        Ok((all_tasks, project_map, project_colors))
    }

    pub async fn get_comments(
        &self,
        task_id: &str,
    ) -> Result<Vec<crate::opus::models::Comment>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/comment/{}", self.base_url, task_id);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to get comments: {} - {}", status, error_text).into())
        }
    }

    pub async fn add_comment_to_task(
        &self,
        task_id: &str,
        content: &str,
    ) -> Result<crate::opus::models::Comment, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/comment/{}", self.base_url, task_id);
        let payload = serde_json::json!({ "content": content });
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to add comment: {} - {}", status, error_text).into())
        }
    }

    pub async fn search(
        &self,
        query: &str,
        search_type: Option<&str>,
    ) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
        let type_param = search_type.unwrap_or("all");
        let encoded_query: String = query
            .chars()
            .map(|c| match c {
                ' ' => "%20".to_string(),
                '&' => "%26".to_string(),
                '=' => "%3D".to_string(),
                '?' => "%3F".to_string(),
                '#' => "%23".to_string(),
                _ if c.is_ascii_alphanumeric() || "-._~".contains(c) => c.to_string(),
                _ => format!("%{:02X}", c as u32),
            })
            .collect();
        let url = format!(
            "{}/api/search/?q={}&type={}",
            self.base_url, encoded_query, type_param
        );
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Search failed: {} - {}", status, error_text).into())
        }
    }

    pub async fn get_tasks_for_filter(
        &self,
        filter_id: &str,
    ) -> Result<Vec<Task>, Box<dyn Error + Send + Sync>> {
        debug_log(&format!("Fetching tasks for filter (column) {}", filter_id));
        let all_tasks = self.get_all_tasks().await?;
        let filtered: Vec<Task> = all_tasks
            .into_iter()
            .filter(|t| t.column_id.as_deref() == Some(filter_id))
            .collect();
        debug_log(&format!(
            "Filter {} matched {} tasks",
            filter_id,
            filtered.len()
        ));
        Ok(filtered)
    }

    pub async fn update_task_with_magic(
        &self,
        task_id: &str,
        magic_text: &str,
    ) -> Result<Task, Box<dyn Error + Send + Sync>> {
        debug_log(&format!(
            "Updating task {} with magic text: '{}'",
            task_id, magic_text
        ));
        let parsed = self.parser.parse(magic_text);

        let update = OpusTaskUpdate {
            title: Some(parsed.title.clone()),
            description: None,
            priority: parsed.priority.clone(),
            due_date: parsed.due_date,
            start_date: parsed.start_date,
            column_id: None,
        };

        let updated_task = self.update_task(task_id, &update).await?;

        for label_name in &parsed.labels {
            match self.ensure_label_exists(label_name).await {
                Ok(label) => {
                    let _ = self.attach_label(&label.id, &updated_task.id).await;
                }
                Err(e) => {
                    debug_log(&format!(
                        "Failed to ensure label '{}' exists: {}",
                        label_name, e
                    ));
                }
            }
        }

        for username in &parsed.assignees {
            if let Some(user) = self.find_user_by_name(username).await {
                let _ = self.update_task_assignee(&updated_task.id, &user.id).await;
            }
        }

        Ok(updated_task)
    }

    pub async fn update_task_from_form(
        &self,
        task_id: &str,
        title: &str,
        description: &str,
        due_date: Option<&str>,
        start_date: Option<&str>,
        priority: Option<&str>,
        label_ids: &[String],
        assignee_ids: &[String],
        comment: Option<&str>,
    ) -> Result<Task, Box<dyn Error + Send + Sync>> {
        debug_log(&format!("Updating task {} from form", task_id));

        let update = OpusTaskUpdate {
            title: Some(title.to_string()),
            description: if description.is_empty() {
                None
            } else {
                Some(description.to_string())
            },
            priority: priority.map(|p| p.to_string()),
            due_date: due_date.and_then(|d| {
                chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                    .ok()
                    .and_then(|nd| nd.and_hms_opt(0, 0, 0))
                    .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            }),
            start_date: start_date.and_then(|d| {
                chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                    .ok()
                    .and_then(|nd| nd.and_hms_opt(0, 0, 0))
                    .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            }),
            column_id: None,
        };

        let _updated_task = self.update_task(task_id, &update).await?;

        let existing_labels = self.get_task_labels(task_id).await.unwrap_or_default();
        let existing_label_ids: Vec<&str> = existing_labels.iter().map(|l| l.id.as_str()).collect();

        for lid in label_ids {
            if !existing_label_ids.contains(&lid.as_str()) {
                let _ = self.attach_label(lid, task_id).await;
            }
        }
        for existing in &existing_labels {
            if !label_ids.iter().any(|lid| lid == &existing.id) {
                let _ = self.detach_label(&existing.id, task_id).await;
            }
        }

        for uid in assignee_ids {
            let _ = self.update_task_assignee(task_id, uid).await;
        }

        if let Some(content) = comment {
            if !content.trim().is_empty() {
                let _ = self.add_comment_to_task(task_id, content).await;
            }
        }

        self.get_task(task_id).await
    }

    pub async fn ensure_label_exists(
        &self,
        label_name: &str,
    ) -> Result<crate::opus::models::Label, Box<dyn Error + Send + Sync>> {
        let labels = self.get_all_labels().await?;
        if let Some(label) = labels
            .into_iter()
            .find(|l| l.name.eq_ignore_ascii_case(label_name))
        {
            return Ok(label);
        }
        self.create_label(label_name, "#808080").await
    }
}

