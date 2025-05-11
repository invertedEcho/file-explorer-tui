pub mod mpsc_utils {
    use core::panic;
    use std::sync::mpsc::Sender;

    pub fn send_message_or_panic(sender: &mut Sender<String>, message: String) {
        let result = sender.send(message);
        match result {
            Ok(_) => {}
            Err(error) => {
                panic!("{:?}", error)
            }
        }
    }
}
