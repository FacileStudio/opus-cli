use crate::opus::models::{Task, Project, Workspace};
use reqwest::Client;
use std::collections::HashMap;

#[derive(Clone)]
pub struct OpusClient {
    client: Client,
    api_url: String,
}

impl OpusClient {
    pub fn new(api_url: String, token: &str) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
                .expect("invalid auth token"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("failed to build HTTP client");

        OpusClient { client, api_url }
    }

    pub async fn get_workspaces(&self) -> Result<Vec<Workspace>, reqwest::Error> {
        let url = format!("{}/workspaces", self.api_url);
        self.client
            .get(&url)
            .send()
            .await?
            .json::<Vec<Workspace>>()
            .await
    }

    pub async fn get_projects(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<Project>, reqwest::Error> {
        let url = format!(
            "{}/workspaces/{}/projects",
            self.api_url, workspace_id
        );
        self.client
            .get(&url)
            .send()
            .await?
            .json::<Vec<Project>>()
            .await
    }

    pub async fn get_tasks(
        &self,
        project_id: &str,
    ) -> Result<Vec<Task>, reqwest::Error> {
        let url = format!("{}/projects/{}/tasks", self.api_url, project_id);
        self.client
            .get(&url)
            .send()
            .await?
            .json::<Vec<Task>>()
            .await
    }

    pub async fn get_task(
        &self,
        project_id: &str,
        task_id: &str,
    ) -> Result<Task, reqwest::Error> {
        let url = format!(
            "{}/projects/{}/tasks/{}",
            self.api_url, project_id, task_id
        );
        self.client
            .get(&url)
            .send()
            .await?
            .json::<Task>()
            .await
    }

    pub async fn get_tasks_with_projects(
        &self,
        workspace_id: &str,
    ) -> Result<(Vec<Task>, HashMap<String, String>), reqwest::Error> {
        let projects = self.get_projects(workspace_id).await?;

        let project_map: HashMap<String, String> = projects
            .iter()
            .map(|p| (p.id.clone(), p.name.clone()))
            .collect();

        let mut all_tasks = Vec::new();
        for project in &projects {
            let tasks = self.get_tasks(&project.id).await?;
            all_tasks.extend(tasks);
        }

        Ok((all_tasks, project_map))
    }
}
