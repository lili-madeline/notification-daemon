mod notification_server;

use crate::notification_server::{start_server, Notification};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

#[tokio::main(flavor = "current_thread")]
async fn main() -> zbus::Result<()> {
    let notification_map = Arc::new(Mutex::new(HashMap::<u32, Notification>::new()));
    let _server = start_server(notification_map.clone()).await?;
    let mut time = SystemTime::now();
    loop {
        if time.elapsed().unwrap_or(Duration::new(0, 0)) >= Duration::new(0, 50000000) {
            let temp = notification_map.lock().unwrap();
            temp.iter().for_each(|(id, notif)| {
                println!("(box :class \"notification\"\n\t:orientation \"v\"\n\t:spacing 0\n\t(box :class \"title\"\n\t\t{}\n\t)\n\t(box :class \"body\"\n\t\t:valign \"start\"\n\t\t:halign \"start\"\n\t\t{}\n\t)\n)",
                    notif.app_name, notif.body,
                );
            });
            time = SystemTime::now();
        }
    }
}
