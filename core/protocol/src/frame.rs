impl Frame {
    pub fn validate(&self) {
        // payload length is implicit in Vec, but protocol users
        // should never construct empty control frames by accident
        match self.frame_type {
            FrameType::Publish | FrameType::Message => {
                assert!(
                    !self.payload.is_empty(),
                    "message frames must carry a payload"
                );
            }
            _ => {}
        }
    }
}
