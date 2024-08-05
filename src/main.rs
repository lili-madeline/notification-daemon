use std::collections::HashMap;
use std::future::pending;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use zbus::export::serde::Deserialize;
use zbus::zvariant::{DeserializeDict, Type};
use zbus::{connection, interface, DBusError};

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
struct Notification {
    app_name: String,
    replaces_id: u32,
    app_icon: String,
    summary: String,
    body: String,
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
    pub notif_map: Arc<Mutex<HashMap<u32, Notification>>>,
}

#[interface(name = "org.freedesktop.Notifications")]
impl Notifications {
    async fn get_capabilities(&self) -> Box<[String]> {
        Box::new(["".to_string()])
    }

    async fn notify(&mut self, notification: Notification) -> u32 {
        let mut notif_map = self.notif_map.lock().unwrap();
        let mut count = 1;
        println!("{:?}", notification);

        if notification.replaces_id != 0 {
            let replaces_id = notification.replaces_id;
            notif_map.remove(&replaces_id);
            notif_map.insert(replaces_id, notification);
            replaces_id
        } else {
            while notif_map.keys().any(|&x| x == count) {
                count += 1;
            }
            notif_map.insert(count, notification);
            count
        }
    }

    async fn close_notification(&mut self, id: u32) -> Result<(), Error> {
        let mut notif_map = self.notif_map.lock().unwrap();
        if notif_map.keys().any(|&x| x == id) {
            notif_map.remove(&id);
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> zbus::Result<()> {
    let notif_map = Arc::new(Mutex::new(HashMap::<u32, Notification>::new()));
    let notification = Notifications {
        notif_map: notif_map.clone(),
    };
    let _connection = connection::Builder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", notification)?
        .build()
        .await?;
    let mut time = SystemTime::now();
    loop {
        if time.elapsed().unwrap_or(Duration::new(0, 0)) >= Duration::new(0, 50000000) {
            println!("{:?}", notif_map.clone().lock().unwrap());
            time = SystemTime::now();
        }
    }
    //pending::<()>().await;
    //Ok(())
}
