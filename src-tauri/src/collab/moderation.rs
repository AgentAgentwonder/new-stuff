use std::collections::{HashMap, HashSet};
use std::time::Duration;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use uuid::Uuid;

use crate::collab::types::{
    ModerationAction, ModerationActionType, ParticipantPermissions, ParticipantRole,
};

#[derive(Default)]
pub struct ModerationManager {
    actions: RwLock<HashMap<Uuid, Vec<ModerationAction>>>,
    banned_users: RwLock<HashSet<String>>,
}

impl ModerationManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_action(&self, action: ModerationAction) {
        if action.action_type == ModerationActionType::Ban {
            self.banned_users
                .write()
                .insert(action.target_user_id.clone());
        }

        let mut map = self.actions.write();
        map.entry(action.room_id).or_default().push(action);
    }

    pub fn is_banned(&self, room_id: &Uuid, user_id: &str) -> bool {
        if self.banned_users.read().contains(user_id) {
            return true;
        }

        let map = self.actions.read();
        if let Some(actions) = map.get(room_id) {
            actions.iter().any(|action| {
                action.action_type == ModerationActionType::Ban
                    && action.target_user_id == user_id
                    && action
                        .expires_at
                        .map(|exp| exp > Utc::now())
                        .unwrap_or(true)
            })
        } else {
            false
        }
    }

    pub fn clean_expired(&self, room_id: &Uuid) {
        let mut map = self.actions.write();
        if let Some(actions) = map.get_mut(room_id) {
            actions.retain(|action| {
                action
                    .expires_at
                    .map(|exp| exp > Utc::now())
                    .unwrap_or(true)
            });
        }
    }

    pub fn apply_moderation(
        &self,
        room_id: Uuid,
        moderator_id: String,
        target_user_id: String,
        action_type: ModerationActionType,
        reason: String,
        duration: Option<Duration>,
    ) -> Result<ModerationAction> {
        let expires_at = duration.map(|d| Utc::now() + chrono::Duration::from_std(d).unwrap());

        let action = ModerationAction {
            id: Uuid::new_v4(),
            room_id,
            moderator_id,
            target_user_id,
            action_type,
            reason,
            timestamp: Utc::now(),
            expires_at,
        };

        self.record_action(action.clone());

        Ok(action)
    }
}

pub fn ensure_moderation_permission(
    moderator_role: ParticipantRole,
    target_role: ParticipantRole,
    permissions: &ParticipantPermissions,
    action: ModerationActionType,
) -> Result<()> {
    if !permissions.can_moderate {
        return Err(anyhow!("Insufficient permissions"));
    }

    match action {
        ModerationActionType::Mute => {
            if !permissions.can_speak {
                return Err(anyhow!("Moderator does not have speaking privileges"));
            }
        }
        ModerationActionType::Kick | ModerationActionType::Ban => {
            if !permissions.can_kick {
                return Err(anyhow!("Moderator cannot remove participants"));
            }
            if moderator_role == target_role {
                return Err(anyhow!("Cannot moderate user with equal role"));
            }
        }
        ModerationActionType::Warning => {
            // warnings are always allowed if can_moderate
        }
    }

    Ok(())
}
