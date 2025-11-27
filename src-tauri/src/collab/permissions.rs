use crate::collab::types::{ParticipantPermissions, ParticipantRole};

pub fn default_permissions_for_role(role: ParticipantRole) -> ParticipantPermissions {
    match role {
        ParticipantRole::Owner => ParticipantPermissions {
            can_speak: true,
            can_share_video: true,
            can_share_screen: true,
            can_chat: true,
            can_share_orders: true,
            can_share_strategies: true,
            can_moderate: true,
            can_kick: true,
            can_ban: true,
        },
        ParticipantRole::Moderator => ParticipantPermissions {
            can_speak: true,
            can_share_video: true,
            can_share_screen: true,
            can_chat: true,
            can_share_orders: true,
            can_share_strategies: true,
            can_moderate: true,
            can_kick: true,
            can_ban: false,
        },
        ParticipantRole::Member => ParticipantPermissions {
            can_speak: true,
            can_share_video: true,
            can_share_screen: true,
            can_chat: true,
            can_share_orders: true,
            can_share_strategies: true,
            can_moderate: false,
            can_kick: false,
            can_ban: false,
        },
        ParticipantRole::Guest => ParticipantPermissions {
            can_speak: false,
            can_share_video: false,
            can_share_screen: false,
            can_chat: false,
            can_share_orders: false,
            can_share_strategies: false,
            can_moderate: false,
            can_kick: false,
            can_ban: false,
        },
    }
}

pub fn can_modify_permissions(role: ParticipantRole) -> bool {
    matches!(role, ParticipantRole::Owner | ParticipantRole::Moderator)
}
