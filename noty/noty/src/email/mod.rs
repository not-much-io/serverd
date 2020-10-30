pub mod smtpgmail;

use lettre::transport::smtp::authentication::Credentials as LettreCredentials;

use nursery_prelude::library_prelude::*;

pub struct Credentials(LettreCredentials);

pub struct Notification {
    message: String,
}

#[async_trait]
pub trait Notify {
    async fn notify(credentials: LettreCredentials, notification: Notification);
}
