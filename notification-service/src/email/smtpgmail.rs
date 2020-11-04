use lettre::transport::smtp::authentication::Credentials;
use lettre::{message::Mailbox, Message, SmtpTransport, Transport};

use crate::email::{Notification, Notify};

use internal_prelude::library_prelude::*;

pub struct SmtpGmail();

#[async_trait]
impl Notify for SmtpGmail {
    async fn notify(credentials: Credentials, notification: Notification) {
        let nmio: Mailbox = "".parse().unwrap();

        let msg = Message::builder()
            .from(nmio.clone())
            .to(nmio)
            .subject("Noty notification")
            .body(notification.message)
            .unwrap();

        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(credentials)
            .build();

        match mailer.send(&msg) {
            Ok(r) => println!("Email sent successfully: {:?}", r),
            Err(e) => panic!("Could not send email: {:?}", e),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test() {
        // let creds = Credentials::new("kristo.koert@gmail.com".into(), "".into());
        // SmtpGmail::notify(
        //     creds,
        //     Notification {
        //         message: "test".to_string(),
        //     },
        // )
        // .await;
    }
}
