# Participant Privacy & Access Control System

## Overview

This document defines the layered participant information display system for HeSOCIAL events, implementing privacy controls and paid access restrictions for viewing other participants.

## Core Principles

1. **Payment-Based Access**: Only paid participants can view other participant information
2. **Privacy Level Control**: Each user controls their visibility through privacy levels (1-5)
3. **Graduated Information Disclosure**: Information visibility increases with payment status and privacy settings
4. **Event-Specific Access**: Participant viewing is limited to specific events the user has paid for

## Access Levels

### Level 1: Public (Non-Participants)
- **Who**: Users not registered for the event
- **Can See**: 
  - Total number of participants
  - General demographics (age ranges, profession categories)
  - No individual participant information

### Level 2: Registered (Unpaid)
- **Who**: Users registered but payment status is 'pending'
- **Can See**:
  - Same as Level 1
  - Confirmation that registration is pending payment

### Level 3: Paid Participants (Basic Access)
- **Who**: Users with payment status 'paid' for the specific event
- **Can See**:
  - Other paid participants based on their privacy levels
  - Participant count breakdown by membership tier
  - Access to participant list with privacy-filtered information

### Level 4: Premium Access (Future Enhancement)
- **Who**: VIP/VVIP tier paid participants
- **Can See**:
  - Enhanced participant information
  - Contact request capabilities
  - Advanced filtering options

## Privacy Levels (User-Controlled)

### Privacy Level 1 (Public Profile)
**Visible to all paid participants:**
- First name + Last initial
- Age range (35-40, 41-45, etc.)
- Profession category (Finance, Tech, Healthcare, etc.)
- Membership tier
- Interests (general categories)
- Profile picture (if set)

### Privacy Level 2 (Semi-Private)
**Visible to paid participants:**
- All Level 1 information
- Full first name
- Company industry (not company name)
- Years of experience range
- City (not full address)

### Privacy Level 3 (Selective Sharing)
**Visible to paid participants:**
- All Level 2 information
- Full name
- Company name (if Fortune 500 or well-known)
- Specific interests and hobbies
- Professional achievements (titles, awards)

### Privacy Level 4 (Enhanced Profile)
**Visible to paid participants:**
- All Level 3 information
- Contact information (platform-mediated)
- Social media links (if professional)
- Detailed bio
- Investment interests

### Privacy Level 5 (Full Disclosure)
**Visible to paid participants:**
- All Level 4 information
- Direct contact information
- Personal interests beyond professional
- Network connections within platform
- Availability for private meetings

## Implementation Logic

### Participant Viewing Access Matrix

```typescript
interface ParticipantViewAccess {
  canViewParticipants: boolean
  maxPrivacyLevelVisible: number
  canSeeContactInfo: boolean
  canInitiateContact: boolean
  participantCountVisible: boolean
}

function getParticipantViewAccess(
  viewerPaymentStatus: PaymentStatus,
  viewerMembershipTier: MembershipTier,
  eventId: string
): ParticipantViewAccess {
  
  // Non-paid participants cannot see participant details
  if (viewerPaymentStatus !== 'paid') {
    return {
      canViewParticipants: false,
      maxPrivacyLevelVisible: 0,
      canSeeContactInfo: false,
      canInitiateContact: false,
      participantCountVisible: true
    }
  }
  
  // Paid participants can see others based on their tier
  const baseAccess = {
    canViewParticipants: true,
    maxPrivacyLevelVisible: 3,
    canSeeContactInfo: false,
    canInitiateContact: true,
    participantCountVisible: true
  }
  
  // Enhanced access for premium tiers
  if (viewerMembershipTier === 'Diamond' || viewerMembershipTier === 'Black Card') {
    baseAccess.maxPrivacyLevelVisible = 5
    baseAccess.canSeeContactInfo = true
  }
  
  return baseAccess
}
```

### Participant Information Filtering

```typescript
interface FilteredParticipantInfo {
  id: string
  displayName: string
  profession?: string
  membershipTier: string
  interests?: string[]
  profilePicture?: string
  ageRange?: string
  city?: string
  company?: string
  bio?: string
  contactInfo?: ContactInfo
  privacyLevel: number
}

function filterParticipantInfo(
  participant: User,
  viewerAccess: ParticipantViewAccess
): FilteredParticipantInfo | null {
  
  // If participant's privacy level exceeds viewer's access, hide completely
  if (participant.privacyLevel > viewerAccess.maxPrivacyLevelVisible) {
    return null
  }
  
  const filtered: FilteredParticipantInfo = {
    id: participant.id,
    displayName: getDisplayName(participant, participant.privacyLevel),
    membershipTier: participant.membershipTier,
    privacyLevel: participant.privacyLevel
  }
  
  // Add information based on privacy level
  if (participant.privacyLevel >= 1) {
    filtered.profession = getProfessionCategory(participant.profession)
    filtered.interests = participant.interests.slice(0, 3) // General categories only
    filtered.profilePicture = participant.profilePicture
    filtered.ageRange = getAgeRange(participant.age)
  }
  
  if (participant.privacyLevel >= 2) {
    filtered.city = participant.city
    filtered.company = getCompanyCategory(participant.company)
  }
  
  if (participant.privacyLevel >= 3) {
    filtered.displayName = `${participant.firstName} ${participant.lastName}`
    filtered.company = participant.company
    filtered.bio = participant.bio?.substring(0, 200) // Truncated bio
  }
  
  if (participant.privacyLevel >= 4 && viewerAccess.canSeeContactInfo) {
    filtered.bio = participant.bio
    filtered.contactInfo = {
      email: participant.email,
      linkedIn: participant.socialMedia?.linkedIn
    }
  }
  
  if (participant.privacyLevel >= 5 && viewerAccess.canSeeContactInfo) {
    filtered.contactInfo = {
      ...filtered.contactInfo,
      phone: participant.phone,
      personalSocial: participant.socialMedia
    }
  }
  
  return filtered
}
```

## Database Schema Updates

### New Tables Required

```sql
-- Event Participant Access Control
CREATE TABLE event_participant_access (
  id TEXT PRIMARY KEY,
  event_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  payment_status TEXT NOT NULL CHECK (payment_status IN ('pending', 'paid', 'refunded')),
  access_level INTEGER NOT NULL DEFAULT 0,
  granted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  expires_at TIMESTAMP,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (event_id) REFERENCES events(id),
  FOREIGN KEY (user_id) REFERENCES users(id),
  UNIQUE(event_id, user_id)
);

-- Participant View Logs (for analytics and abuse prevention)
CREATE TABLE participant_view_logs (
  id TEXT PRIMARY KEY,
  viewer_id TEXT NOT NULL,
  viewed_participant_id TEXT NOT NULL,
  event_id TEXT NOT NULL,
  view_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  view_type TEXT NOT NULL CHECK (view_type IN ('list', 'profile', 'contact')),
  FOREIGN KEY (viewer_id) REFERENCES users(id),
  FOREIGN KEY (viewed_participant_id) REFERENCES users(id),
  FOREIGN KEY (event_id) REFERENCES events(id)
);

-- User Privacy Preferences (extends users table)
ALTER TABLE users ADD COLUMN default_privacy_level INTEGER DEFAULT 2;
ALTER TABLE users ADD COLUMN show_in_participant_lists BOOLEAN DEFAULT true;
ALTER TABLE users ADD COLUMN allow_contact_requests BOOLEAN DEFAULT true;
```

## API Endpoints

### Get Event Participants
```
GET /api/events/:eventId/participants
Authorization: Bearer <token>
Query Parameters:
  - page: number (pagination)
  - limit: number (results per page)
  - membershipTier: string (filter)
  - profession: string (filter)
  - privacyLevel: number (minimum privacy level)

Response:
{
  "success": true,
  "data": {
    "participants": FilteredParticipantInfo[],
    "totalCount": number,
    "paidParticipantCount": number,
    "viewerAccess": ParticipantViewAccess
  },
  "pagination": PaginationInfo
}
```

### Check Participant Access
```
GET /api/events/:eventId/participant-access
Authorization: Bearer <token>

Response:
{
  "success": true,
  "data": {
    "hasAccess": boolean,
    "accessLevel": ParticipantViewAccess,
    "paymentRequired": boolean,
    "paymentStatus": string
  }
}
```

## UI Components Structure

### ParticipantList Component
- Displays filtered participant cards
- Handles pagination and filtering
- Shows access restrictions messaging
- Integrates with payment flow

### ParticipantCard Component
- Displays participant information based on privacy level
- Handles contact initiation
- Shows privacy level indicators
- Responsive design for mobile

### PaymentGate Component
- Blocks participant access for unpaid users
- Clear messaging about payment requirements
- Integration with existing payment system
- Upgrade prompts for enhanced access

## Security Considerations

1. **Server-Side Filtering**: All participant information filtering must happen server-side
2. **Access Logging**: Log all participant viewing activities for abuse monitoring
3. **Rate Limiting**: Prevent bulk scraping of participant information
4. **Privacy Compliance**: Ensure GDPR/CCPA compliance for data sharing
5. **Audit Trail**: Maintain logs of privacy level changes and access grants

## Privacy Controls for Users

### User Settings Page
- Default privacy level selection
- Event-specific privacy overrides
- Contact request preferences
- Visibility in participant lists toggle
- Data sharing consent management

### Event-Specific Privacy
Users can override their default privacy level for specific events:
```typescript
interface EventPrivacyOverride {
  eventId: string
  privacyLevel: number
  allowContact: boolean
  showInList: boolean
  validUntil: Date
}
```

## Future Enhancements

1. **Premium Participant Features**
   - Enhanced profiles for Diamond/Black Card members
   - Private messaging within platform
   - Meeting scheduling integration

2. **Social Networking Features**
   - Connection requests
   - Follow/unfollow participants
   - Post-event networking tools

3. **AI-Powered Matching**
   - Interest-based participant suggestions
   - Business networking recommendations
   - Conversation starter suggestions

4. **Analytics Dashboard**
   - Organizer insights on participant engagement
   - Privacy level distribution analytics
   - Connection success metrics