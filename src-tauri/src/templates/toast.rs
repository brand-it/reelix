use crate::templates::InlineTemplate;
use askama::Template;

#[derive(Clone, Debug)]
pub struct Toast {
    pub title: String,
    pub message: String,
    pub variant: ToastVariant,
    #[allow(dead_code)]
    pub auto_hide_ms: u32,
    pub action_link: Option<ToastAction>,
}

#[derive(Clone, Debug)]
pub struct ToastAction {
    pub label: String,
    pub href: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Copy)]
pub enum ToastVariant {
    Success,
    Danger,
    #[allow(dead_code)]
    Warning,
    #[allow(dead_code)]
    Info,
}

impl ToastVariant {
    pub fn bg_class(&self) -> &'static str {
        match self {
            ToastVariant::Success => "bg-success",
            ToastVariant::Danger => "bg-danger",
            ToastVariant::Warning => "bg-warning",
            ToastVariant::Info => "bg-info",
        }
    }

    pub fn text_class(&self) -> &'static str {
        "text-white"
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ToastVariant::Success => "fas fa-check-circle",
            ToastVariant::Danger => "fas fa-exclamation-circle",
            ToastVariant::Warning => "fas fa-exclamation-triangle",
            ToastVariant::Info => "fas fa-info-circle",
        }
    }
}

impl Toast {
    pub fn new(
        title: impl Into<String>,
        message: impl Into<String>,
        variant: ToastVariant,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            variant,
            auto_hide_ms: 5000,
            action_link: None,
        }
    }

    pub fn success(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message, ToastVariant::Success)
    }

    pub fn danger(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message, ToastVariant::Danger)
    }

    #[allow(dead_code)]
    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message, ToastVariant::Warning)
    }

    #[allow(dead_code)]
    pub fn info(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message, ToastVariant::Info)
    }

    pub fn with_auto_hide(mut self, ms: u32) -> Self {
        self.auto_hide_ms = ms;
        self
    }

    pub fn with_action(mut self, label: impl Into<String>, href: impl Into<String>) -> Self {
        self.action_link = Some(ToastAction {
            label: label.into(),
            href: href.into(),
        });
        self
    }

    pub fn id(&self) -> String {
        // Generate a simple ID based on title and message hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.title.hash(&mut hasher);
        self.message.hash(&mut hasher);
        format!("toast-{}", hasher.finish())
    }
}

#[derive(Template)]
#[template(path = "toast/item.html")]
pub struct ToastItem {
    pub toast: Toast,
}

#[derive(Template)]
#[template(path = "toast/append.turbo.html")]
pub struct ToastAppend<'a> {
    pub item: &'a ToastItem,
}

pub fn render_toast_append(toast: Toast) -> Result<String, crate::templates::Error> {
    let item = ToastItem { toast };
    let template = ToastAppend { item: &item };
    crate::templates::render(template)
}
