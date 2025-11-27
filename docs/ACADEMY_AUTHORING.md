# Academy Content Authoring Guide

## Overview

The Academy system provides a comprehensive platform for creating educational content, tracking user progress, issuing rewards, and managing certifications. This guide covers how to create and manage academy content.

## Architecture

### Backend Components

The Academy backend is organized into three main modules:

1. **Content Service** (`src-tauri/src/academy/content.rs`)
   - Course and lesson management
   - Quiz and challenge creation
   - Webinar scheduling
   - Mentor profiles

2. **Progress Tracker** (`src-tauri/src/academy/progress.rs`)
   - User enrollment and progress tracking
   - Quiz attempts and grading
   - Challenge submissions
   - Attendance records

3. **Reward Engine** (`src-tauri/src/academy/rewards.rs`)
   - Badge creation and issuance
   - Certificate generation
   - XP and reward calculation
   - Reputation integration

### Frontend Components

Located in `src/components/academy/` and `src/pages/Academy.tsx`:

- Course catalog and viewer
- Progress dashboards
- Leaderboard displays
- Mentor matching interface
- Badge and certificate displays

## Creating Content

### 1. Creating a Course

Courses are the top-level educational units. Each course contains multiple lessons and can award badges upon completion.

```typescript
import { invoke } from '@tauri-apps/api/tauri';

const course = {
  id: 'trading-basics-101',
  title: 'Trading Basics 101',
  description: 'Learn the fundamentals of trading on Solana',
  level: 'beginner', // beginner | intermediate | advanced | expert
  category: 'trading',
  durationMinutes: 180,
  xpReward: 1000,
  badgeId: 'trading_beginner', // optional
  prerequisites: [], // array of prerequisite course IDs
  tags: ['trading', 'solana', 'basics'],
  thumbnailUrl: 'https://example.com/image.png', // optional
  isPublished: true,
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
};

await invoke('create_course', { course });
```

### 2. Creating Lessons

Lessons are individual learning units within a course. They can be various content types.

```typescript
const lesson = {
  id: 'lesson-intro-to-trading',
  courseId: 'trading-basics-101',
  title: 'Introduction to Trading',
  description: 'Understand what trading is and basic concepts',
  contentType: 'video', // video | tutorial | article | quiz | challenge | webinar | livesession
  contentUrl: 'https://youtube.com/watch?v=...', // for video/webinar
  contentData: JSON.stringify({
    // For interactive content
    sections: [
      { type: 'text', content: 'Trading is...' },
      { type: 'image', url: 'https://...' },
    ],
  }),
  orderIndex: 1,
  durationMinutes: 15,
  xpReward: 50,
  isMandatory: true,
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
};

await invoke('create_lesson', { lesson });
```

### 3. Creating Quizzes

Quizzes test learner knowledge and award XP based on performance.

```typescript
const quiz = {
  id: 'quiz-trading-basics',
  lessonId: 'lesson-intro-to-trading',
  title: 'Trading Basics Quiz',
  questions: [
    {
      id: 'q1',
      question: 'What is a limit order?',
      options: [
        'An order to buy/sell at a specific price',
        'An order to buy/sell at market price',
        'An order that never expires',
        'An order that auto-executes',
      ],
      correctAnswer: 0,
      explanation: 'A limit order specifies the maximum or minimum price at which you are willing to buy or sell.',
      points: 10,
    },
    // Add more questions...
  ],
  passingScore: 70, // Percentage
  maxAttempts: 3, // null for unlimited
  timeLimitMinutes: 30, // null for no limit
};

await invoke('create_quiz', { quiz });
```

### 4. Creating Challenges

Challenges are practical tasks that learners complete to demonstrate mastery.

```typescript
const challenge = {
  id: 'challenge-first-trade',
  title: 'Make Your First Trade',
  description: 'Execute your first successful trade on the platform',
  category: 'trading',
  difficulty: 'beginner',
  xpReward: 500,
  badgeId: 'first_trade_badge',
  requirements: JSON.stringify({
    action: 'complete_trade',
    minAmount: 10,
    tokenType: 'SOL',
  }),
  validationCriteria: JSON.stringify({
    verifyTransaction: true,
    checkSuccess: true,
  }),
  startDate: new Date().toISOString(), // optional
  endDate: null, // optional, null = ongoing
  createdAt: new Date().toISOString(),
};

await invoke('create_challenge', { challenge });
```

### 5. Scheduling Webinars

Webinars are live or recorded sessions with instructors.

```typescript
const webinar = {
  id: 'webinar-advanced-strategies',
  title: 'Advanced Trading Strategies',
  description: 'Learn advanced trading techniques from experts',
  instructor: 'John Doe',
  scheduledAt: new Date('2024-12-01T18:00:00Z').toISOString(),
  durationMinutes: 90,
  maxParticipants: 100, // null for unlimited
  meetingUrl: 'https://zoom.us/j/...', // optional
  recordingUrl: null, // filled after webinar
  xpReward: 200,
  status: 'scheduled', // scheduled | live | completed | cancelled
  createdAt: new Date().toISOString(),
};

await invoke('create_webinar', { webinar });
```

### 6. Creating Mentor Profiles

Mentors provide one-on-one guidance to learners.

```typescript
const mentor = {
  id: 'mentor-jane-smith',
  walletAddress: '7xKXtg2CW...',
  name: 'Jane Smith',
  bio: 'Professional trader with 10 years of experience...',
  expertiseAreas: ['trading', 'defi', 'risk-management'],
  languages: ['English', 'Spanish'],
  availability: JSON.stringify({
    timezone: 'UTC-5',
    hours: [
      { day: 'Monday', start: '09:00', end: '17:00' },
      { day: 'Wednesday', start: '09:00', end: '17:00' },
    ],
  }),
  rating: 5.0,
  totalSessions: 0,
  isActive: true,
  createdAt: new Date().toISOString(),
};

await invoke('create_mentor', { mentor });
```

### 7. Creating Badges

Badges are achievements that learners earn for completing specific tasks.

```typescript
const badge = {
  id: 'trading_master',
  name: 'Trading Master',
  description: 'Complete all trading courses with excellent scores',
  rarity: 'legendary', // common | uncommon | rare | epic | legendary
  iconUrl: 'https://example.com/badges/trading_master.png',
  xpReward: 2000,
  reputationBoost: 20.0, // Added to user's reputation score
  requirements: JSON.stringify({
    coursesCompleted: 5,
    category: 'trading',
    minAverageScore: 90,
  }),
  isActive: true,
  createdAt: new Date().toISOString(),
};

await invoke('create_badge', { badge });
```

## Tracking Progress

### Enrolling Users in Courses

```typescript
const progress = await invoke('start_course', {
  walletAddress: '7xKXtg2CW...',
  courseId: 'trading-basics-101',
});
```

### Tracking Lesson Progress

```typescript
// Start a lesson
await invoke('start_lesson', {
  walletAddress: '7xKXtg2CW...',
  lessonId: 'lesson-intro-to-trading',
  courseId: 'trading-basics-101',
});

// Update progress (e.g., video position)
await invoke('update_lesson_progress', {
  walletAddress: '7xKXtg2CW...',
  lessonId: 'lesson-intro-to-trading',
  timeSpent: 15, // minutes
  lastPosition: '00:10:30', // video timestamp or scroll position
});

// Complete a lesson
await invoke('complete_lesson', {
  walletAddress: '7xKXtg2CW...',
  lessonId: 'lesson-intro-to-trading',
  courseId: 'trading-basics-101',
});
```

### Submitting Quiz Attempts

```typescript
const attempt = {
  id: `attempt_${Date.now()}`,
  walletAddress: '7xKXtg2CW...',
  quizId: 'quiz-trading-basics',
  score: 85,
  totalPoints: 100,
  passed: true,
  answers: JSON.stringify([
    { questionId: 'q1', selectedAnswer: 0, correct: true },
    // ...
  ]),
  timeTakenMinutes: 25,
  attemptedAt: new Date().toISOString(),
};

const result = await invoke('submit_quiz', { attempt });
// XP is automatically calculated and awarded
```

### Submitting Challenges

```typescript
const submission = {
  id: `submission_${Date.now()}`,
  walletAddress: '7xKXtg2CW...',
  challengeId: 'challenge-first-trade',
  submissionData: JSON.stringify({
    transactionSignature: '5KJdv...',
    timestamp: new Date().toISOString(),
    amount: 10.5,
  }),
  status: 'pending', // pending | approved | rejected
  score: null,
  feedback: null,
  submittedAt: new Date().toISOString(),
  reviewedAt: null,
};

await invoke('submit_challenge', { submission });
```

## Reward Calculation

### XP Calculation Formulas

#### Course Completion
```
base_xp = duration_minutes * 10
xp = base_xp * difficulty_multiplier

Multipliers:
- beginner: 1.0
- intermediate: 1.5
- advanced: 2.0
- expert: 2.5
```

#### Quiz Performance
```
base_xp = score * 5
bonus = 0 (if < 80%)
      = base_xp * 0.25 (if 80-89%)
      = base_xp * 0.50 (if >= 90%)
time_bonus = base_xp * 0.1 (if completed quickly)

total_xp = base_xp + bonus + time_bonus
```

#### Challenge Completion
```
base_xp (by difficulty):
- beginner: 500
- intermediate: 1000
- advanced: 2000
- expert: 5000

total_xp = base_xp * completion_quality (0.0-1.0)
```

### Awarding Badges

Badges are awarded automatically when users meet requirements or manually by administrators.

```typescript
const earnedBadge = await invoke('award_badge', {
  walletAddress: '7xKXtg2CW...',
  badgeId: 'trading_master',
  source: 'Completed all trading courses',
});
// XP and reputation boost are automatically applied
```

### Issuing Certificates

```typescript
const certificate = {
  id: `cert_${Date.now()}`,
  walletAddress: '7xKXtg2CW...',
  courseId: 'trading-basics-101', // or null
  challengeId: null, // or null
  title: 'Trading Basics Certification',
  description: 'Successfully completed Trading Basics 101 with distinction',
  issuedAt: new Date().toISOString(),
  certificateUrl: 'https://certificates.example.com/cert123.pdf',
  verificationCode: 'CERT-2024-1234567',
};

await invoke('issue_certificate', { certificate });
```

## User Statistics and Leaderboard

### Getting User Stats

```typescript
const userStats = await invoke('get_user_stats', {
  walletAddress: '7xKXtg2CW...',
});

// Returns:
// {
//   totalCoursesEnrolled: 5,
//   totalCoursesCompleted: 3,
//   totalLessonsCompleted: 25,
//   totalQuizzesPassed: 15,
//   totalChallengesCompleted: 8,
//   totalXp: 15000,
//   currentStreakDays: 7,
//   longestStreakDays: 14,
//   badgesEarned: ['first_lesson', 'quiz_master', ...],
// }
```

### Leaderboard

```typescript
const leaderboard = await invoke('get_leaderboard', { limit: 100 });

// Returns top users by XP:
// [
//   {
//     walletAddress: '...',
//     rank: 1,
//     totalXp: 50000,
//     coursesCompleted: 15,
//     badgesCount: 12,
//     streakDays: 30,
//   },
//   ...
// ]
```

## Best Practices

### Content Design

1. **Structure Courses Progressively**: Start with basic concepts and gradually increase difficulty
2. **Use Mixed Content Types**: Combine videos, articles, quizzes, and hands-on challenges
3. **Set Clear Prerequisites**: Define course dependencies to ensure proper learning paths
4. **Provide Immediate Feedback**: Include explanations in quiz questions
5. **Make Content Bite-Sized**: Keep lessons under 20 minutes for better retention

### Reward Design

1. **Balance XP Awards**: Ensure rewards scale appropriately with difficulty and time investment
2. **Create Milestone Badges**: Recognize achievements at key progression points
3. **Offer Exclusive Rewards**: High-value badges should require significant effort
4. **Tie to Reputation**: Use reputation boosts to incentivize quality participation
5. **Gamify Progress**: Use streaks, leaderboards, and challenges to maintain engagement

### Quality Assurance

1. **Test All Quizzes**: Verify that correct answers and scoring work properly
2. **Review Challenge Validation**: Ensure challenges can be verified automatically or manually
3. **Check Prerequisites**: Confirm that prerequisite chains don't create circular dependencies
4. **Verify Reward Math**: Test XP calculations with various scenarios
5. **Validate Content**: Review all content for accuracy, clarity, and accessibility

## Integration with Reputation System

The Academy integrates with the platform's reputation system:

1. **Badge Rewards Include Reputation Boosts**: Each badge adds to the user's overall reputation
2. **Course Completion Affects Trust Score**: Successfully completing courses contributes to reputation
3. **Quality Participation Matters**: High quiz scores and challenge completions increase reputation
4. **Mentor Ratings Feed Reputation**: Mentor sessions with high ratings boost both parties' reputation

## API Reference

### Content Commands
- `create_course` - Create a new course
- `get_course` - Get course details
- `list_courses` - List all published courses with optional filters
- `create_lesson` - Add a lesson to a course
- `get_course_lessons` - Get all lessons for a course
- `create_quiz` - Create a quiz for a lesson
- `get_quiz` - Get quiz details
- `create_challenge` - Create a challenge
- `list_challenges` - List active challenges
- `create_webinar` - Schedule a webinar
- `list_webinars` - List webinars by status
- `create_mentor` - Add a mentor profile
- `list_mentors` - List available mentors
- `get_content_stats` - Get overall content statistics

### Progress Commands
- `start_course` - Enroll user in a course
- `get_user_progress` - Get user's course progress
- `complete_course` - Mark course as completed and award XP/badges
- `start_lesson` - Begin a lesson
- `get_lesson_progress` - Get lesson progress
- `update_lesson_progress` - Update lesson progress (time, position)
- `complete_lesson` - Mark lesson as completed
- `submit_quiz` - Submit a quiz attempt
- `get_quiz_attempts` - Get all attempts for a quiz
- `submit_challenge` - Submit a challenge solution
- `get_challenge_submissions` - Get user's challenge submissions
- `record_webinar_attendance` - Record webinar attendance
- `create_mentor_session` - Schedule a mentor session
- `get_user_mentor_sessions` - Get user's mentor sessions
- `get_user_stats` - Get comprehensive user statistics
- `get_leaderboard` - Get top users by XP

### Reward Commands
- `create_badge` - Create a new badge
- `get_badge` - Get badge details
- `list_badges` - List all active badges
- `award_badge` - Award a badge to a user
- `get_user_badges` - Get user's earned badges
- `issue_certificate` - Issue a certificate
- `get_user_certificates` - Get user's certificates
- `verify_certificate` - Verify a certificate by code
- `get_user_rewards` - Get user's rewards (claimed/unclaimed)
- `claim_reward` - Claim a specific reward
- `claim_all_rewards` - Claim all pending rewards
- `get_reward_stats` - Get reward system statistics

## Support

For questions or issues:
- Check the inline code documentation
- Review test files in `src/__tests__/academy/`
- See `REPUTATION_SYSTEM.md` for reputation integration details
