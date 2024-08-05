use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use zbus::zvariant::{DeserializeDict, Type};
use zbus::{connection, interface, Connection, DBusError};

#[derive(DeserializeDict, Type, Debug)]
#[zvariant(signature = "a{sv}")]
#[allow(unused)]
struct Hint {
    name: Option<String>,
    variant: Option<Variant>,
}

#[derive(DeserializeDict, Debug, Type)]
#[zvariant(signature = "v")]
#[allow(unused)]
struct Variant {
    boolean: Option<bool>,
    string: Option<Box<String>>,
    iiibiiay: Option<Box<[u8]>>,
    int32: Option<i32>,
    byte: Option<u8>,
}

#[derive(Deserialize, Type, Debug)]
pub(crate) struct Notification {
    pub(crate) app_name: String,
    replaces_id: u32,
    pub(crate) app_icon: String,
    pub(crate) summary: String,
    pub(crate) body: String,
    actions: Box<[String]>,
    hints: Hint,
    expire_timeout: i32,
}

#[derive(Debug, DBusError)]
enum Error {
    #[zbus(error)]
    ZBus(zbus::Error),
}

struct Notifications {
    pub notification_map: Arc<Mutex<HashMap<u32, Notification>>>,
}

#[interface(name = "org.freedesktop.Notifications")]
impl Notifications {
    async fn get_capabilities(&self) -> Box<[String]> {
        Box::new([
            "icon-static".to_string(),
            "body".to_string(),
            "body-images".to_string(),
            "persistence".to_string(),
        ])
    }

    async fn notify(&mut self, notification: Notification) -> u32 {
        let mut notification_map = self.notification_map.lock().unwrap();
        let mut count = 1;
        println!("{:?}", notification);

        if notification.replaces_id != 0 {
            let replaces_id = notification.replaces_id;
            notification_map.remove(&replaces_id);
            notification_map.insert(replaces_id, notification);
            replaces_id
        } else {
            while notification_map.keys().any(|&x| x == count) {
                count += 1;
            }
            notification_map.insert(count, notification);
            count
        }
    }

    async fn close_notification(&mut self, id: u32) -> Result<(), Error> {
        let mut notification_map = self.notification_map.lock().unwrap();
        if notification_map.keys().any(|&x| x == id) {
            notification_map.remove(&id);
            Ok(())
        } else {
            Err(Error::ZBus(zbus::Error::Failure("".to_string())))
        }
    }

    async fn get_server_information(&self) -> (String, String, String, String) {
        (
            "lili".to_string(),
            "lili".to_string(),
            "0.1".to_string(),
            "1.2".to_string(),
        )
    }

    async fn notification_closed(&mut self, id: u32, reason: u32) {}

    async fn action_invoked(&mut self, id: u32, action_key: String) {}

    async fn activation_token(&mut self, id: u32, activation_token: u32) {}
}

pub(crate) async fn start_server(
    notification_map: Arc<Mutex<HashMap<u32, Notification>>>,
) -> zbus::Result<Connection> {
    let notification = Notifications {
        notification_map: notification_map.clone(),
    };
    let connection = connection::Builder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", notification)?
        .build()
        .await?;
    Ok(connection)
}
