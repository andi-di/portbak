use super::Expression;
use uuid::Uuid;

#[derive(Clone, Copy)]
pub enum Action {
    AutomaticDisabled,
    AutomaticEnabled,
    CheckNow,
    DarkMode,
    Failed,
    LightMode,
    Maximized,
    NewJob,
    ProtectionEnabled,
    RemoveJob,
    RestoreCompleted,
    RestoreConfirmation,
}

impl Action {
    pub(super) fn pool(self) -> &'static [Expression] {
        use Expression::*;
        match self {
            Self::AutomaticDisabled => &[BoredExpectant, Pouting],
            Self::AutomaticEnabled => &[Dominant, Energetic, Happy],
            Self::CheckNow => &[Dominant, Energetic],
            Self::DarkMode => &[Happy, Smug],
            Self::Failed => &[Sad],
            Self::LightMode => &[Angry, Flustered, Sleepy],
            Self::Maximized => &[Angry, Excited, Flustered, Happy, Pouting],
            Self::NewJob => &[Energetic, Excited, Happy],
            Self::ProtectionEnabled => &[Dominant, Killer],
            Self::RemoveJob => &[Meek, Pouting, Sad],
            Self::RestoreCompleted => &[Celebrating, Happy, Smug],
            Self::RestoreConfirmation => &[Praying, Question],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Backups(Uuid),
    JobSettings(Uuid),
    SettingsJobs,
    SettingsTheme,
    StartScreen,
}

impl Page {
    pub(super) fn pool(self) -> &'static [Expression] {
        use Expression::*;
        match self {
            Self::Backups(_) => &[Dominant, Praying, Smug],
            Self::JobSettings(_) => &[BoredExpectant, Curious, Dominant, Smug],
            Self::SettingsJobs => &[Curious, Question, Waiting],
            Self::SettingsTheme => &[Cute, Flirty, Seductive],
            Self::StartScreen => &[BoredExpectant, Pleading, Waiting],
        }
    }
}
