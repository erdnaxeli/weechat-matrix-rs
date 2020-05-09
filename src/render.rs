use matrix_sdk::events::room::{
    encrypted::EncryptedEvent,
    member::{MemberEvent, MembershipState},
    message::{
        AudioMessageEventContent, EmoteMessageEventContent,
        FileMessageEventContent, ImageMessageEventContent, MessageEvent,
        MessageEventContent, NoticeMessageEventContent,
        TextMessageEventContent, VideoMessageEventContent,
    },
};

/// This trait describes events that can be rendered in the weechat UI
pub(crate) trait RenderableEvent {
    /// Convert the event into a string that will be displayed in the UI.
    /// The displayname is taken as a parameter since it cannot be calculated from the event
    /// context alone.
    fn render(&self, displayname: &str) -> String;
}

impl RenderableEvent for EncryptedEvent {
    // TODO: this is not implemented yet
    fn render(&self, displayname: &str) -> String {
        format!("{}\t{}", displayname, "Unable to decrypt message")
    }
}

impl RenderableEvent for MemberEvent {
    fn render(&self, displayname: &str) -> String {
        let operation = match self.content.membership {
            MembershipState::Join => "joined",
            MembershipState::Leave => "left",
            MembershipState::Ban => "banned",
            MembershipState::Invite => "invited",
            MembershipState::Knock => "knocked on", // TODO
        };
        format!(
            "{} ({}) has {} the room",
            displayname, self.state_key, operation
        )
    }
}

impl RenderableEvent for MessageEvent {
    fn render(&self, displayname: &str) -> String {
        use MessageEventContent::*;

        match &self.content {
            Text(t) => format!("{}\t{}", displayname, t.resolve_body()),
            Emote(e) => format!("{}\t{}", displayname, e.resolve_body()),
            Audio(a) => {
                format!("{}\t{}: {}", displayname, a.body, a.resolve_url())
            }
            File(f) => {
                format!("{}\t{}: {}", displayname, f.body, f.resolve_url())
            }
            Image(i) => {
                format!("{}\t{}: {}", displayname, i.body, i.resolve_url())
            }
            Location(l) => {
                format!("{}\t{}: {}", displayname, l.body, l.geo_uri)
            }
            Notice(n) => format!("{}\t{}", displayname, n.resolve_body()),
            Video(v) => {
                format!("{}\t{}: {}", displayname, v.body, v.resolve_url())
            }
            ServerNotice(sn) => {
                format!("SERVER\t{}", sn.body) // TODO
            }
        }
    }
}

/// Trait for message event types that contain an optional formatted body. `resolve_body` will
/// return the formatted body if present, else fallback to the regular body.
trait HasFormattedBody {
    fn body(&self) -> &str;
    fn formatted_body(&self) -> Option<&str>;
    #[inline]
    fn resolve_body(&self) -> &str {
        self.formatted_body().unwrap_or_else(|| self.body())
    }
}

// Repeating this for each event type would get boring fast so lets use a simple macro to implement
// the trait for a struct that has a `body` and `formatted_body` field
macro_rules! has_formatted_body {
    ($content: ident) => {
        impl HasFormattedBody for $content {
            #[inline]
            fn body(&self) -> &str {
                &self.body
            }

            #[inline]
            fn formatted_body(&self) -> Option<&str> {
                self.formatted_body.as_deref()
            }
        }
    };
}

/// This trait is implemented for message types that can contain either an URL or an encrypted
/// file. One of both _must_ be present.
trait HasUrlOrFile {
    fn url(&self) -> Option<&str>;
    fn file(&self) -> Option<&str>;
    #[inline]
    fn resolve_url(&self) -> &str {
        // the file is either encrypted or not encrypted so either `url` or `file` must
        // exist and unwrapping will never panic
        self.url().or_else(|| self.file()).unwrap()
    }
}

// Same as above: a simple macro to implement the trait for structs with `url` and `file` fields.
macro_rules! has_url_or_file {
    ($content: ident) => {
        impl HasUrlOrFile for $content {
            #[inline]
            fn url(&self) -> Option<&str> {
                self.url.as_deref()
            }

            #[inline]
            fn file(&self) -> Option<&str> {
                self.file.as_ref().map(|f| f.url.as_str())
            }
        }
    };
}

// this actually implements the trait for different event types
has_formatted_body!(EmoteMessageEventContent);
has_formatted_body!(NoticeMessageEventContent);
has_formatted_body!(TextMessageEventContent);

has_url_or_file!(AudioMessageEventContent);
has_url_or_file!(FileMessageEventContent);
has_url_or_file!(ImageMessageEventContent);
has_url_or_file!(VideoMessageEventContent);
