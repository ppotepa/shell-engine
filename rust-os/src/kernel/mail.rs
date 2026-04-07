use crate::state::MailMessage;

pub struct MailSpool {
    messages: Vec<MailMessage>,
}

impl Default for MailSpool {
    fn default() -> Self {
        Self::new()
    }
}

impl MailSpool {
    pub fn new() -> Self {
        let mut spool = Self {
            messages: Vec::new(),
        };
        spool.seed_initial_mail();
        spool
    }

    fn seed_initial_mail(&mut self) {
        self.messages.push(MailMessage {
            from: "op@kruuna".to_string(),
            to: "torvalds@kruuna".to_string(),
            subject: "welcome".to_string(),
            body: "you made it in. good.\n\nthe system is running minix 1.1.\naccounts are limited. use resources wisely.\n\n— op".to_string(),
            date: "Mon, 16 Sep 1991 18:42:00 +0300".to_string(),
            read: false,
        });
        self.messages.push(MailMessage {
            from: "ast@cs.vu.nl".to_string(),
            to: "torvalds@kruuna".to_string(),
            subject: "Re: your kernel".to_string(),
            body: "Linus,\n\nInteresting project. I looked at the code.\nOne note on the file transfer: compressed archives (.Z files)\nmust be transferred in BINARY mode. ASCII mode corrupts the\ncompression headers. This is not negotiable.\n\n— ast".to_string(),
            date: "Mon, 16 Sep 1991 20:11:00 +0200".to_string(),
            read: false,
        });
    }

    pub fn list(&self) -> &[MailMessage] {
        &self.messages
    }

    pub fn get(&self, index: usize) -> Option<&MailMessage> {
        self.messages.get(index)
    }

    pub fn mark_read(&mut self, index: usize) {
        if let Some(m) = self.messages.get_mut(index) {
            m.read = true;
        }
    }

    pub fn deliver(&mut self, from: &str, to: &str, subject: &str, body: &str, date: &str) {
        self.messages.push(MailMessage {
            from: from.to_string(),
            to: to.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
            date: date.to_string(),
            read: false,
        });
    }

    pub fn unread_count(&self) -> usize {
        self.messages.iter().filter(|m| !m.read).count()
    }
}
