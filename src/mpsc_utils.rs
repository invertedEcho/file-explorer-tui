pub mod mpsc_utils {
    use std::sync::mpsc::Sender;

    use log::warn;

    pub fn send_message_or_panic(sender: &mut Sender<String>, message: String) {
        let result = sender.send(message);
        match result {
            Ok(_) => {}
            Err(error) => {
                warn!("Failed to send message via mpsc: {:?}", error);
            }
        }
    }
}
