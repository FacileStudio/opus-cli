use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use crate::opus::models::Attachment;

pub struct AttachmentViewer {
    pub attachments: Vec<Attachment>,
    pub selected_index: usize,
}

impl AttachmentViewer {
    pub fn new(attachments: Vec<Attachment>) -> Self {
        Self {
            attachments,
            selected_index: 0,
        }
    }

    pub fn draw(&self, f: &mut Frame, area: Rect) {
        if self.attachments.is_empty() {
            let block = Block::default()
                .title("Attachments")
                .borders(Borders::ALL);
            let text = Paragraph::new("No attachments")
                .block(block)
                .style(Style::default().fg(Color::Gray));
            f.render_widget(text, area);
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ])
            .split(area);

        self.draw_attachment_list(f, chunks[0]);
        self.draw_attachment_preview(f, chunks[1]);
    }

    fn draw_attachment_list(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Attachments")
            .borders(Borders::ALL);

        let mut lines = Vec::new();

        for (i, attachment) in self.attachments.iter().enumerate() {
            let is_selected = i == self.selected_index;
            let style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let file_name = &attachment.filename;
            let size_text = self.format_file_size(attachment.size);

            let icon = self.get_file_icon(file_name, attachment.mime_type.as_deref());

            lines.push(Line::from(vec![
                Span::styled(icon, style),
                Span::styled(" ", style),
                Span::styled(file_name.as_str(), style),
                Span::styled(format!(" ({})", size_text), style.fg(Color::Gray)),
            ]));
        }

        let text = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(text, area);
    }

    fn draw_attachment_preview(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Preview")
            .borders(Borders::ALL);

        if self.attachments.is_empty() {
            let text = Paragraph::new("No attachment selected")
                .block(block)
                .style(Style::default().fg(Color::Gray));
            f.render_widget(text, area);
            return;
        }

        let attachment = &self.attachments[self.selected_index];
        let file_name = &attachment.filename;

        if self.is_image_file(file_name, attachment.mime_type.as_deref()) {
            self.draw_image_preview(f, area, block, attachment);
        } else {
            self.draw_file_info(f, area, block, attachment);
        }
    }

    fn draw_image_preview(&self, f: &mut Frame, area: Rect, block: Block, attachment: &Attachment) {
        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("Image Preview", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        ]));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![
            Span::styled("File: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(&attachment.filename, Style::default().fg(Color::Blue))
        ]));

        if let Some(mime) = &attachment.mime_type {
            lines.push(Line::from(vec![
                Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(mime, Style::default().fg(Color::Yellow))
            ]));
        }

        if let Some(size) = attachment.size {
            lines.push(Line::from(vec![
                Span::styled("Size: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(self.format_file_size(Some(size)), Style::default().fg(Color::Cyan))
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from("Press 'd' to download"));
        lines.push(Line::from("Press 'r' to remove"));
        lines.push(Line::from(""));
        lines.push(Line::from("Note: Full image preview requires"));
        lines.push(Line::from("downloading the attachment first."));

        let text = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(text, area);
    }

    fn draw_file_info(&self, f: &mut Frame, area: Rect, block: Block, attachment: &Attachment) {
        let mut lines = Vec::new();

        let file_name = &attachment.filename;
        let icon = self.get_file_icon(file_name, attachment.mime_type.as_deref());

        lines.push(Line::from(vec![
            Span::styled(icon, Style::default().fg(Color::Yellow)),
            Span::styled(" ", Style::default()),
            Span::styled(file_name.as_str(), Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
        ]));
        lines.push(Line::from(""));

        if let Some(mime) = &attachment.mime_type {
            lines.push(Line::from(vec![
                Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(mime, Style::default().fg(Color::Yellow))
            ]));
        }

        if let Some(size) = attachment.size {
            lines.push(Line::from(vec![
                Span::styled("Size: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(self.format_file_size(Some(size)), Style::default().fg(Color::Cyan))
            ]));
        }

        if let Some(created) = &attachment.created_at {
            lines.push(Line::from(vec![
                Span::styled("Created: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(created.format("%Y-%m-%d %H:%M").to_string(), Style::default().fg(Color::Green))
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from("Press 'd' to download"));
        lines.push(Line::from("Press 'r' to remove"));
        lines.push(Line::from("Press 'u' to upload new file"));

        let text = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(text, area);
    }

    fn format_file_size(&self, size: Option<i64>) -> String {
        match size {
            Some(size) => {
                if size > 1024 * 1024 * 1024 {
                    format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
                } else if size > 1024 * 1024 {
                    format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
                } else if size > 1024 {
                    format!("{:.1} KB", size as f64 / 1024.0)
                } else {
                    format!("{} bytes", size)
                }
            }
            None => "Unknown size".to_string(),
        }
    }

    fn is_image_file(&self, filename: &str, mime_type: Option<&str>) -> bool {
        let image_extensions = [
            "jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico", "tiff", "tga"
        ];

        if let Some(ext) = filename.split('.').last() {
            if image_extensions.iter().any(|&img_ext| ext.eq_ignore_ascii_case(img_ext)) {
                return true;
            }
        }

        if let Some(mime) = mime_type {
            if mime.starts_with("image/") {
                return true;
            }
        }

        false
    }

    fn get_file_icon(&self, filename: &str, mime_type: Option<&str>) -> &'static str {
        if self.is_image_file(filename, mime_type) {
            "IMG"
        } else {
            let ext = filename.split('.').last().unwrap_or("").to_lowercase();
            match ext.as_str() {
                "pdf" => "PDF",
                "txt" | "md" | "rst" => "TXT",
                "doc" | "docx" => "DOC",
                "xls" | "xlsx" => "XLS",
                "ppt" | "pptx" => "PPT",
                "zip" | "rar" | "7z" | "tar" | "gz" => "ZIP",
                "mp3" | "wav" | "flac" => "AUD",
                "mp4" | "avi" | "mov" => "VID",
                "py" | "js" | "rs" | "go" | "java" | "cpp" | "c" => "SRC",
                "json" | "xml" | "yaml" | "yml" => "CFG",
                _ => "FILE",
            }
        }
    }

    pub fn next_attachment(&mut self) {
        if !self.attachments.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.attachments.len();
        }
    }

    pub fn previous_attachment(&mut self) {
        if !self.attachments.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.attachments.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn get_selected_attachment(&self) -> Option<&Attachment> {
        self.attachments.get(self.selected_index)
    }
}
