use resend_rs::Resend;

#[derive(Clone)]
pub struct ResendClient {
    resend: Resend,
}

impl ResendClient {
    pub fn new() -> Self {
        let resend = Resend::default();
        ResendClient { resend }
    }
}
