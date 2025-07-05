import passport from 'passport'
import { Strategy as GoogleStrategy } from 'passport-google-oauth20'
import { Strategy as JwtStrategy, ExtractJwt } from 'passport-jwt'
import { v4 as uuidv4 } from 'uuid'
import { duckdb } from '../database/duckdb-connection.js'
import { User } from '../types/index.js'
import config from '../utils/config.js'
import logger from '../utils/logger.js'

// Google OAuth Strategy
passport.use(
  new GoogleStrategy(
    {
      clientID: config.oauth.google.clientId,
      clientSecret: config.oauth.google.clientSecret,
      callbackURL: '/api/auth/google/callback'
    },
    async (accessToken, refreshToken, profile, done) => {
      try {
        const email = profile.emails?.[0]?.value
        if (!email) {
          return done(new Error('No email found in Google profile'), null)
        }

        // Check if user already exists
        const existingUserQuery = 'SELECT * FROM users WHERE email = $1 AND deleted_at IS NULL'
        const existingUser = await duckdb.query(existingUserQuery, [email])

        if (existingUser.rows.length > 0) {
          // User exists, update Google info if needed
          const user = existingUser.rows[0]
          
          // Update Google profile info
          const updateQuery = `
            UPDATE users SET
              profile_picture = COALESCE($2, profile_picture),
              updated_at = $3
            WHERE id = $1
          `
          
          await duckdb.query(updateQuery, [
            user.id,
            profile.photos?.[0]?.value,
            new Date().toISOString()
          ])

          // Get updated user data
          const userQuery = `
            SELECT 
              id, email, first_name as "firstName", last_name as "lastName",
              age, profession, annual_income as "annualIncome", net_worth as "netWorth",
              membership_tier as "membershipTier", privacy_level as "privacyLevel",
              is_verified as "isVerified", verification_status as "verificationStatus",
              role, profile_picture as "profilePicture", bio, interests,
              created_at as "createdAt", updated_at as "updatedAt"
            FROM users 
            WHERE id = $1
          `
          
          const userResult = await duckdb.query(userQuery, [user.id])
          return done(null, userResult.rows[0] as User)
        } else {
          // Create new user from Google profile
          const userId = uuidv4()
          const firstName = profile.name?.givenName || profile.displayName?.split(' ')[0] || ''
          const lastName = profile.name?.familyName || profile.displayName?.split(' ').slice(1).join(' ') || ''
          
          const insertUserQuery = `
            INSERT INTO users (
              id, email, first_name, last_name, age, profession,
              annual_income, net_worth, membership_tier, privacy_level,
              is_verified, verification_status, profile_picture, bio, interests,
              created_at, updated_at
            ) VALUES (
              $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17
            )
          `

          await duckdb.query(insertUserQuery, [
            userId,
            email,
            firstName,
            lastName,
            null, // age - to be filled by user
            null, // profession - to be filled by user
            null, // annual_income - to be filled by user
            null, // net_worth - to be filled by user
            'Platinum', // default membership tier
            3, // default privacy level
            false, // needs verification
            'pending',
            profile.photos?.[0]?.value || null,
            null, // bio
            JSON.stringify([]), // empty interests array
            new Date().toISOString(),
            new Date().toISOString()
          ])

          // Get created user
          const userQuery = `
            SELECT 
              id, email, first_name as "firstName", last_name as "lastName",
              age, profession, annual_income as "annualIncome", net_worth as "netWorth",
              membership_tier as "membershipTier", privacy_level as "privacyLevel",
              is_verified as "isVerified", verification_status as "verificationStatus",
              role, profile_picture as "profilePicture", bio, interests,
              created_at as "createdAt", updated_at as "updatedAt"
            FROM users 
            WHERE id = $1
          `
          
          const userResult = await duckdb.query(userQuery, [userId])
          const newUser = userResult.rows[0] as User

          logger.info(`New user created via Google OAuth: ${email}`)
          return done(null, newUser)
        }
      } catch (error) {
        logger.error('Google OAuth error:', error)
        return done(error, null)
      }
    }
  )
)

// JWT Strategy
passport.use(
  new JwtStrategy(
    {
      jwtFromRequest: ExtractJwt.fromAuthHeaderAsBearerToken(),
      secretOrKey: config.jwtSecret
    },
    async (payload, done) => {
      try {
        const userQuery = `
          SELECT 
            id, email, first_name as "firstName", last_name as "lastName",
            age, profession, annual_income as "annualIncome", net_worth as "netWorth",
            membership_tier as "membershipTier", privacy_level as "privacyLevel",
            is_verified as "isVerified", verification_status as "verificationStatus",
            role, profile_picture as "profilePicture", bio, interests,
            created_at as "createdAt", updated_at as "updatedAt"
          FROM users 
          WHERE id = $1 AND deleted_at IS NULL
        `
        
        const result = await duckdb.query(userQuery, [payload.userId])
        
        if (result.rows.length === 0) {
          return done(null, false)
        }

        return done(null, result.rows[0] as User)
      } catch (error) {
        logger.error('JWT strategy error:', error)
        return done(error, false)
      }
    }
  )
)

// Serialize/deserialize for session support (if needed)
passport.serializeUser((user: any, done) => {
  done(null, user.id)
})

passport.deserializeUser(async (id: string, done) => {
  try {
    const userQuery = `
      SELECT 
        id, email, first_name as "firstName", last_name as "lastName",
        age, profession, annual_income as "annualIncome", net_worth as "netWorth",
        membership_tier as "membershipTier", privacy_level as "privacyLevel",
        is_verified as "isVerified", verification_status as "verificationStatus",
        role, profile_picture as "profilePicture", bio, interests,
        created_at as "createdAt", updated_at as "updatedAt"
      FROM users 
      WHERE id = $1 AND deleted_at IS NULL
    `
    
    const result = await duckdb.query(userQuery, [id])
    
    if (result.rows.length === 0) {
      return done(null, false)
    }

    done(null, result.rows[0] as User)
  } catch (error) {
    logger.error('Deserialize user error:', error)
    done(error, false)
  }
})

export default passport