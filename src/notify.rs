use anyhow::Result;
use tracing::debug;

/// Cross-platform notifier — uses OS commands, zero extra deps.
#[derive(Clone)]
pub struct Notifier {
    pub desktop: bool,
    pub bell: bool,
}

impl Notifier {
    pub fn new(desktop: bool, bell: bool) -> Self {
        Self { desktop, bell }
    }

    pub async fn send(&self, title: &str, body: &str) -> Result<()> {
        if self.bell {
            print!("\x07");
        }
        if self.desktop {
            self.send_desktop(title, body);
        }
        debug!("Notification: {} — {}", title, body);
        Ok(())
    }

    fn send_desktop(&self, title: &str, body: &str) {
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("notify-send")
                .arg("--expire-time=6000")
                .arg(title)
                .arg(body)
                .spawn();
        }

        #[cfg(target_os = "macos")]
        {
            let script = format!(
                "display notification \"{}\" with title \"{}\"",
                body.replace('"', "\\\""),
                title.replace('"', "\\\"")
            );
            let _ = std::process::Command::new("osascript")
                .args(["-e", &script])
                .spawn();
        }

        #[cfg(target_os = "windows")]
        {
            let script = format!(
                "[void][Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType=WindowsRuntime]; \
                 $t=[Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent(0); \
                 $t.SelectSingleNode('//text[@id=1]').InnerText='{}'; \
                 $n=[Windows.UI.Notifications.ToastNotification]::new($t); \
                 [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Binance-Claw').Show($n)",
                title.replace('\'', "")
            );
            let _ = std::process::Command::new("powershell")
                .args(["-WindowStyle", "Hidden", "-Command", &script])
                .spawn();
        }

        // Suppress unused variable warnings on non-target platforms
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            let _ = (title, body);
        }
    }
}
