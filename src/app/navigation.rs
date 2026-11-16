use super::App;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Page {
    #[default]
    Home,
    JobSettings(Uuid),
    Backups(Uuid),
    Settings,
}

impl Page {
    pub(super) fn for_job(id: Option<Uuid>) -> Self {
        id.map_or(Self::Home, Self::JobSettings)
    }

    pub(super) fn selected_job(self) -> Option<Uuid> {
        match self {
            Self::JobSettings(id) | Self::Backups(id) => Some(id),
            _ => None,
        }
    }

    pub(super) fn is_settings(self) -> bool {
        self == Self::Settings
    }
    pub(super) fn is_backups(self) -> bool {
        matches!(self, Self::Backups(_))
    }
}

impl App {
    pub(super) fn toggle_settings(&mut self) {
        self.page = if self.page.is_settings() {
            Page::Home
        } else {
            Page::Settings
        };
    }

    pub(super) fn toggle_job(&mut self, id: Uuid) {
        self.page = if self.page.selected_job() == Some(id) {
            Page::Home
        } else {
            Page::JobSettings(id)
        };
    }
}
