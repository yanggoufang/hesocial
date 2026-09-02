#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    User,
    Admin,
    SuperAdmin,
}

impl Role {
    pub fn parse(value: Option<&str>) -> Option<Self> {
        match value.map(str::trim).filter(|s| !s.is_empty()) {
            Some("user") => Some(Self::User),
            Some("admin") => Some(Self::Admin),
            Some("super_admin") => Some(Self::SuperAdmin),
            _ => None,
        }
    }

    fn level(self) -> u8 {
        match self {
            Self::User => 1,
            Self::Admin => 2,
            Self::SuperAdmin => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MembershipTier {
    Platinum,
    Diamond,
    BlackCard,
}

impl MembershipTier {
    pub fn parse(value: Option<&str>) -> Option<Self> {
        match value.map(str::trim).filter(|s| !s.is_empty()) {
            Some("Platinum") => Some(Self::Platinum),
            Some("Diamond") => Some(Self::Diamond),
            Some("Black Card") => Some(Self::BlackCard),
            _ => None,
        }
    }

    fn level(self) -> u8 {
        match self {
            Self::Platinum => 1,
            Self::Diamond => 2,
            Self::BlackCard => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationStatus {
    Pending,
    Approved,
    Rejected,
}

impl VerificationStatus {
    pub fn parse(value: Option<&str>) -> Option<Self> {
        match value.map(str::trim).filter(|s| !s.is_empty()) {
            Some("pending") => Some(Self::Pending),
            Some("approved") => Some(Self::Approved),
            Some("rejected") => Some(Self::Rejected),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AuthSnapshot {
    pub is_authenticated: bool,
    pub role: Option<Role>,
    pub membership_tier: Option<MembershipTier>,
    pub is_verified: bool,
    pub verification_status: Option<VerificationStatus>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Can {
    pub access: bool,
    pub view_admin: bool,
    pub manage_super_admin: bool,
    pub manage_users: bool,
    pub manage_events: bool,
    pub manage_venues: bool,
    pub manage_categories: bool,
    pub manage_backups: bool,
    pub view_sales_data: bool,
    pub access_vvip: bool,
    pub access_premium_events: bool,
    pub access_exclusive_events: bool,
    pub upload_media: bool,
    pub register_for_events: bool,
    pub access_private_content: bool,
    pub full_admin_access: bool,
    pub event_management: bool,
    pub member_features: bool,
}

pub fn permissions(snapshot: &AuthSnapshot) -> Can {
    let role_level = snapshot.role.map(Role::level).unwrap_or(0);
    let is_admin = role_level >= 2;
    let is_super_admin = role_level >= 3;
    let is_diamond = snapshot.membership_tier.map(MembershipTier::level).unwrap_or(0) >= 2;
    let is_black_card = snapshot.membership_tier.map(MembershipTier::level).unwrap_or(0) >= 3;
    let is_verified = snapshot.is_verified;

    Can {
        access: snapshot.is_authenticated,
        view_admin: is_admin,
        manage_super_admin: is_super_admin,
        manage_users: is_admin,
        manage_events: is_admin,
        manage_venues: is_admin,
        manage_categories: is_admin,
        manage_backups: is_super_admin,
        view_sales_data: is_admin,
        access_vvip: is_diamond && is_verified,
        access_premium_events: is_diamond,
        access_exclusive_events: is_black_card,
        upload_media: snapshot.is_authenticated,
        register_for_events: is_verified,
        access_private_content: is_verified,
        full_admin_access: is_super_admin,
        event_management: is_admin,
        member_features: snapshot.is_authenticated && is_verified,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AuthUser {
    pub id: Option<String>,
    pub email: Option<String>,
    pub role: Option<Role>,
    pub membership_tier: Option<MembershipTier>,
    pub is_verified: bool,
    pub verification_status: Option<VerificationStatus>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Session {
    pub token: Option<String>,
    pub user: Option<AuthUser>,
}

impl Session {
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    pub fn snapshot(&self) -> AuthSnapshot {
        match &self.user {
            Some(user) => user.snapshot(self.is_authenticated()),
            None => AuthSnapshot {
                is_authenticated: self.is_authenticated(),
                ..AuthSnapshot::default()
            },
        }
    }

    pub fn view_admin(&self) -> bool {
        permissions(&self.snapshot()).view_admin
    }
}

impl AuthUser {
    pub fn from_json(value: &serde_json::Value) -> Self {
        Self {
            id: value
                .get("id")
                .and_then(|v| v.as_str().map(str::to_string).or_else(|| v.as_i64().map(|n| n.to_string()))),
            email: value.get("email").and_then(|v| v.as_str()).map(str::to_string),
            role: Role::parse(value.get("role").and_then(|v| v.as_str())),
            membership_tier: MembershipTier::parse(value.get("membershipTier").and_then(|v| v.as_str())),
            is_verified: value.get("isVerified").and_then(|v| v.as_bool()).unwrap_or(false),
            verification_status: VerificationStatus::parse(
                value.get("verificationStatus").and_then(|v| v.as_str()),
            ),
        }
    }

    pub fn snapshot(&self, is_authenticated: bool) -> AuthSnapshot {
        AuthSnapshot {
            is_authenticated,
            role: self.role,
            membership_tier: self.membership_tier,
            is_verified: self.is_verified,
            verification_status: self.verification_status,
        }
    }
}
