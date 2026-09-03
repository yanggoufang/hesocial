export interface User {
  id: string;
  email: string;
  firstName: string;
  lastName: string;
  age: number;
  profession: string;
  annualIncome: number;
  netWorth: number;
  membershipTier: 'Platinum' | 'Diamond' | 'Black Card';
  privacyLevel: 1 | 2 | 3 | 4 | 5;
  isVerified: boolean;
  profilePicture?: string;
  bio?: string;
  interests: string[];
  createdAt: string;
  updatedAt: string;
}

export interface Event {
  id: string;
  name: string;
  description: string;
  dateTime: string;
  registrationDeadline: string;
  venue: Venue;
  category: EventCategory;
  organizer: string;
  pricing: EventPricing;
  exclusivityLevel: 'VIP' | 'VVIP' | 'Invitation Only';
  dressCode: 1 | 2 | 3 | 4 | 5;
  capacity: number;
  currentAttendees: number;
  amenities: string[];
  privacyGuarantees: string[];
  images: string[];
  videoUrl?: string;
  requirements: EventRequirement[];
  createdAt: string;
  updatedAt: string;
}

export interface Venue {
  id: string;
  name: string;
  address: string;
  city: string;
  coordinates: {
    lat: number;
    lng: number;
  };
  rating: number;
  amenities: string[];
  images: string[];
}

export interface EventCategory {
  id: string;
  name: string;
  description: string;
  icon: string;
}

export interface EventPricing {
  vip?: number;
  vvip?: number;
  general?: number;
  currency: 'TWD' | 'USD';
  installmentOptions?: number[];
}

export interface EventRequirement {
  type: 'age' | 'income' | 'membership' | 'verification';
  value: string | number;
  description: string;
}

export interface Registration {
  id: string;
  userId: string;
  eventId: string;
  status: 'pending' | 'approved' | 'rejected' | 'cancelled';
  paymentStatus: 'pending' | 'paid' | 'refunded';
  specialRequests?: string;
  createdAt: string;
  updatedAt: string;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}