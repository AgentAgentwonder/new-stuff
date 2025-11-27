use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProgressStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProgress {
    pub id: String,
    pub wallet_address: String,
    pub course_id: String,
    pub status: ProgressStatus,
    pub progress_percentage: f64,
    pub completed_lessons: Vec<String>,
    pub quiz_attempts: i64,
    pub last_accessed: DateTime<Utc>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonProgress {
    pub id: String,
    pub wallet_address: String,
    pub lesson_id: String,
    pub course_id: String,
    pub status: ProgressStatus,
    pub time_spent_minutes: i64,
    pub completion_percentage: f64,
    pub last_position: Option<String>, // For video timestamp or scroll position
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizAttempt {
    pub id: String,
    pub wallet_address: String,
    pub quiz_id: String,
    pub score: i64,
    pub total_points: i64,
    pub passed: bool,
    pub answers: String, // JSON
    pub time_taken_minutes: i64,
    pub attempted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeSubmission {
    pub id: String,
    pub wallet_address: String,
    pub challenge_id: String,
    pub submission_data: String, // JSON
    pub status: String,          // pending, approved, rejected
    pub score: Option<i64>,
    pub feedback: Option<String>,
    pub submitted_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebinarAttendance {
    pub id: String,
    pub wallet_address: String,
    pub webinar_id: String,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub duration_minutes: i64,
    pub engagement_score: f64, // 0-100
    pub certificate_issued: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MentorSession {
    pub id: String,
    pub student_address: String,
    pub mentor_id: String,
    pub topic: String,
    pub scheduled_at: DateTime<Utc>,
    pub duration_minutes: i64,
    pub status: String, // scheduled, completed, cancelled
    pub notes: Option<String>,
    pub student_rating: Option<f64>,
    pub mentor_rating: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStats {
    pub wallet_address: String,
    pub total_courses_enrolled: i64,
    pub total_courses_completed: i64,
    pub total_lessons_completed: i64,
    pub total_quizzes_passed: i64,
    pub total_challenges_completed: i64,
    pub total_webinars_attended: i64,
    pub total_mentor_sessions: i64,
    pub total_xp: i64,
    pub current_streak_days: i64,
    pub longest_streak_days: i64,
    pub badges_earned: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardEntry {
    pub wallet_address: String,
    pub rank: i64,
    pub total_xp: i64,
    pub courses_completed: i64,
    pub badges_count: i64,
    pub streak_days: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum ProgressError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid data: {0}")]
    InvalidData(String),
}

pub struct ProgressTracker {
    pool: Pool<Sqlite>,
}

impl ProgressTracker {
    pub async fn new(app_handle: &AppHandle) -> Result<Self, ProgressError> {
        let app_dir = app_handle.path().app_data_dir().map_err(|err| {
            ProgressError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Unable to resolve app data directory: {err}"),
            ))
        })?;

        let _ = std::fs::create_dir_all(&app_dir);
        let db_path = app_dir.join("academy.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let pool = match SqlitePool::connect(&db_url).await {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!("Warning: ProgressTracker failed to connect to {:?}: {}", db_path, e);
                eprintln!("Falling back to in-memory database for ProgressTracker");
                eprintln!("ProgressTracker using in-memory database for this session");
                SqlitePool::connect("sqlite::memory:").await?
            }
        };

        Self::init_schema(&pool).await?;

        Ok(Self { pool })
    }

    async fn init_schema(pool: &Pool<Sqlite>) -> Result<(), ProgressError> {
        // User progress table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_progress (
                id TEXT PRIMARY KEY NOT NULL,
                wallet_address TEXT NOT NULL,
                course_id TEXT NOT NULL,
                status TEXT NOT NULL,
                progress_percentage REAL NOT NULL DEFAULT 0.0,
                completed_lessons TEXT NOT NULL, -- JSON array
                quiz_attempts INTEGER NOT NULL DEFAULT 0,
                last_accessed TEXT NOT NULL,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                UNIQUE(wallet_address, course_id)
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Lesson progress table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS lesson_progress (
                id TEXT PRIMARY KEY NOT NULL,
                wallet_address TEXT NOT NULL,
                lesson_id TEXT NOT NULL,
                course_id TEXT NOT NULL,
                status TEXT NOT NULL,
                time_spent_minutes INTEGER NOT NULL DEFAULT 0,
                completion_percentage REAL NOT NULL DEFAULT 0.0,
                last_position TEXT,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                UNIQUE(wallet_address, lesson_id)
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Quiz attempts table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS quiz_attempts (
                id TEXT PRIMARY KEY NOT NULL,
                wallet_address TEXT NOT NULL,
                quiz_id TEXT NOT NULL,
                score INTEGER NOT NULL,
                total_points INTEGER NOT NULL,
                passed INTEGER NOT NULL,
                answers TEXT NOT NULL, -- JSON
                time_taken_minutes INTEGER NOT NULL,
                attempted_at TEXT NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Challenge submissions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS challenge_submissions (
                id TEXT PRIMARY KEY NOT NULL,
                wallet_address TEXT NOT NULL,
                challenge_id TEXT NOT NULL,
                submission_data TEXT NOT NULL, -- JSON
                status TEXT NOT NULL DEFAULT 'pending',
                score INTEGER,
                feedback TEXT,
                submitted_at TEXT NOT NULL,
                reviewed_at TEXT
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Webinar attendance table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS webinar_attendance (
                id TEXT PRIMARY KEY NOT NULL,
                wallet_address TEXT NOT NULL,
                webinar_id TEXT NOT NULL,
                joined_at TEXT NOT NULL,
                left_at TEXT,
                duration_minutes INTEGER NOT NULL DEFAULT 0,
                engagement_score REAL NOT NULL DEFAULT 0.0,
                certificate_issued INTEGER NOT NULL DEFAULT 0,
                UNIQUE(wallet_address, webinar_id)
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Mentor sessions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS mentor_sessions (
                id TEXT PRIMARY KEY NOT NULL,
                student_address TEXT NOT NULL,
                mentor_id TEXT NOT NULL,
                topic TEXT NOT NULL,
                scheduled_at TEXT NOT NULL,
                duration_minutes INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'scheduled',
                notes TEXT,
                student_rating REAL,
                mentor_rating REAL,
                created_at TEXT NOT NULL,
                completed_at TEXT
            )
            "#,
        )
        .execute(pool)
        .await?;

        // User stats table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_stats (
                wallet_address TEXT PRIMARY KEY NOT NULL,
                total_xp INTEGER NOT NULL DEFAULT 0,
                current_streak_days INTEGER NOT NULL DEFAULT 0,
                longest_streak_days INTEGER NOT NULL DEFAULT 0,
                last_activity_date TEXT NOT NULL,
                badges_earned TEXT NOT NULL DEFAULT '[]' -- JSON array
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Create indexes
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_user_progress_wallet ON user_progress(wallet_address)",
        )
        .execute(pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_lesson_progress_wallet ON lesson_progress(wallet_address)")
            .execute(pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_stats_xp ON user_stats(total_xp DESC)")
            .execute(pool)
            .await?;

        Ok(())
    }

    // Course progress operations
    pub async fn start_course(
        &self,
        wallet_address: &str,
        course_id: &str,
    ) -> Result<UserProgress, ProgressError> {
        let id = format!("{}_{}", wallet_address, course_id);
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT OR IGNORE INTO user_progress (
                id, wallet_address, course_id, status, progress_percentage,
                completed_lessons, quiz_attempts, last_accessed, started_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(wallet_address)
        .bind(course_id)
        .bind("inprogress")
        .bind(0.0)
        .bind("[]")
        .bind(0)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        self.update_user_activity(wallet_address).await?;

        self.get_user_progress(wallet_address, course_id).await
    }

    pub async fn get_user_progress(
        &self,
        wallet_address: &str,
        course_id: &str,
    ) -> Result<UserProgress, ProgressError> {
        let row =
            sqlx::query("SELECT * FROM user_progress WHERE wallet_address = ? AND course_id = ?")
                .bind(wallet_address)
                .bind(course_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|_| ProgressError::NotFound(format!("Progress not found")))?;

        Self::user_progress_from_row(&row)
    }

    pub async fn update_course_progress(
        &self,
        wallet_address: &str,
        course_id: &str,
        completed_lesson_id: &str,
    ) -> Result<(), ProgressError> {
        // Get current progress
        let progress = self.get_user_progress(wallet_address, course_id).await?;
        let mut completed_lessons = progress.completed_lessons;

        if !completed_lessons.contains(&completed_lesson_id.to_string()) {
            completed_lessons.push(completed_lesson_id.to_string());
        }

        let completed_json = serde_json::to_string(&completed_lessons)?;

        sqlx::query(
            r#"
            UPDATE user_progress 
            SET completed_lessons = ?, last_accessed = ?
            WHERE wallet_address = ? AND course_id = ?
            "#,
        )
        .bind(completed_json)
        .bind(Utc::now().to_rfc3339())
        .bind(wallet_address)
        .bind(course_id)
        .execute(&self.pool)
        .await?;

        self.update_user_activity(wallet_address).await?;

        Ok(())
    }

    pub async fn complete_course(
        &self,
        wallet_address: &str,
        course_id: &str,
    ) -> Result<(), ProgressError> {
        sqlx::query(
            r#"
            UPDATE user_progress 
            SET status = ?, progress_percentage = ?, completed_at = ?
            WHERE wallet_address = ? AND course_id = ?
            "#,
        )
        .bind("completed")
        .bind(100.0)
        .bind(Utc::now().to_rfc3339())
        .bind(wallet_address)
        .bind(course_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Lesson progress operations
    pub async fn start_lesson(
        &self,
        wallet_address: &str,
        lesson_id: &str,
        course_id: &str,
    ) -> Result<LessonProgress, ProgressError> {
        let id = format!("{}_{}", wallet_address, lesson_id);
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO lesson_progress (
                id, wallet_address, lesson_id, course_id, status, 
                time_spent_minutes, completion_percentage, started_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(wallet_address)
        .bind(lesson_id)
        .bind(course_id)
        .bind("inprogress")
        .bind(0)
        .bind(0.0)
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        self.update_user_activity(wallet_address).await?;

        self.get_lesson_progress(wallet_address, lesson_id).await
    }

    pub async fn get_lesson_progress(
        &self,
        wallet_address: &str,
        lesson_id: &str,
    ) -> Result<LessonProgress, ProgressError> {
        let row =
            sqlx::query("SELECT * FROM lesson_progress WHERE wallet_address = ? AND lesson_id = ?")
                .bind(wallet_address)
                .bind(lesson_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|_| ProgressError::NotFound(format!("Lesson progress not found")))?;

        Self::lesson_progress_from_row(&row)
    }

    pub async fn update_lesson_progress(
        &self,
        wallet_address: &str,
        lesson_id: &str,
        time_spent: i64,
        last_position: Option<String>,
    ) -> Result<(), ProgressError> {
        sqlx::query(
            r#"
            UPDATE lesson_progress 
            SET time_spent_minutes = time_spent_minutes + ?, last_position = ?
            WHERE wallet_address = ? AND lesson_id = ?
            "#,
        )
        .bind(time_spent)
        .bind(last_position)
        .bind(wallet_address)
        .bind(lesson_id)
        .execute(&self.pool)
        .await?;

        self.update_user_activity(wallet_address).await?;

        Ok(())
    }

    pub async fn complete_lesson(
        &self,
        wallet_address: &str,
        lesson_id: &str,
        course_id: &str,
    ) -> Result<(), ProgressError> {
        sqlx::query(
            r#"
            UPDATE lesson_progress 
            SET status = ?, completion_percentage = ?, completed_at = ?
            WHERE wallet_address = ? AND lesson_id = ?
            "#,
        )
        .bind("completed")
        .bind(100.0)
        .bind(Utc::now().to_rfc3339())
        .bind(wallet_address)
        .bind(lesson_id)
        .execute(&self.pool)
        .await?;

        // Update course progress
        self.update_course_progress(wallet_address, course_id, lesson_id)
            .await?;

        Ok(())
    }

    // Quiz operations
    pub async fn submit_quiz(&self, attempt: QuizAttempt) -> Result<QuizAttempt, ProgressError> {
        sqlx::query(
            r#"
            INSERT INTO quiz_attempts (
                id, wallet_address, quiz_id, score, total_points, 
                passed, answers, time_taken_minutes, attempted_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&attempt.id)
        .bind(&attempt.wallet_address)
        .bind(&attempt.quiz_id)
        .bind(attempt.score)
        .bind(attempt.total_points)
        .bind(attempt.passed)
        .bind(&attempt.answers)
        .bind(attempt.time_taken_minutes)
        .bind(attempt.attempted_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        self.update_user_activity(&attempt.wallet_address).await?;

        Ok(attempt)
    }

    pub async fn get_quiz_attempts(
        &self,
        wallet_address: &str,
        quiz_id: &str,
    ) -> Result<Vec<QuizAttempt>, ProgressError> {
        let rows = sqlx::query(
            "SELECT * FROM quiz_attempts WHERE wallet_address = ? AND quiz_id = ? ORDER BY attempted_at DESC"
        )
        .bind(wallet_address)
        .bind(quiz_id)
        .fetch_all(&self.pool)
        .await?;

        let mut attempts = Vec::new();
        for row in rows {
            attempts.push(Self::quiz_attempt_from_row(&row)?);
        }

        Ok(attempts)
    }

    // Challenge operations
    pub async fn submit_challenge(
        &self,
        submission: ChallengeSubmission,
    ) -> Result<ChallengeSubmission, ProgressError> {
        sqlx::query(
            r#"
            INSERT INTO challenge_submissions (
                id, wallet_address, challenge_id, submission_data,
                status, submitted_at
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&submission.id)
        .bind(&submission.wallet_address)
        .bind(&submission.challenge_id)
        .bind(&submission.submission_data)
        .bind(&submission.status)
        .bind(submission.submitted_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        self.update_user_activity(&submission.wallet_address)
            .await?;

        Ok(submission)
    }

    pub async fn get_challenge_submissions(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<ChallengeSubmission>, ProgressError> {
        let rows = sqlx::query(
            "SELECT * FROM challenge_submissions WHERE wallet_address = ? ORDER BY submitted_at DESC"
        )
        .bind(wallet_address)
        .fetch_all(&self.pool)
        .await?;

        let mut submissions = Vec::new();
        for row in rows {
            submissions.push(Self::challenge_submission_from_row(&row)?);
        }

        Ok(submissions)
    }

    // Webinar attendance
    pub async fn record_webinar_attendance(
        &self,
        attendance: WebinarAttendance,
    ) -> Result<WebinarAttendance, ProgressError> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO webinar_attendance (
                id, wallet_address, webinar_id, joined_at, left_at,
                duration_minutes, engagement_score, certificate_issued
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&attendance.id)
        .bind(&attendance.wallet_address)
        .bind(&attendance.webinar_id)
        .bind(attendance.joined_at.to_rfc3339())
        .bind(attendance.left_at.map(|d| d.to_rfc3339()))
        .bind(attendance.duration_minutes)
        .bind(attendance.engagement_score)
        .bind(attendance.certificate_issued)
        .execute(&self.pool)
        .await?;

        self.update_user_activity(&attendance.wallet_address)
            .await?;

        Ok(attendance)
    }

    // Mentor sessions
    pub async fn create_mentor_session(
        &self,
        session: MentorSession,
    ) -> Result<MentorSession, ProgressError> {
        sqlx::query(
            r#"
            INSERT INTO mentor_sessions (
                id, student_address, mentor_id, topic, scheduled_at,
                duration_minutes, status, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&session.id)
        .bind(&session.student_address)
        .bind(&session.mentor_id)
        .bind(&session.topic)
        .bind(session.scheduled_at.to_rfc3339())
        .bind(session.duration_minutes)
        .bind(&session.status)
        .bind(session.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn get_user_mentor_sessions(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<MentorSession>, ProgressError> {
        let rows = sqlx::query(
            "SELECT * FROM mentor_sessions WHERE student_address = ? ORDER BY scheduled_at DESC",
        )
        .bind(wallet_address)
        .fetch_all(&self.pool)
        .await?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(Self::mentor_session_from_row(&row)?);
        }

        Ok(sessions)
    }

    // User stats operations
    pub async fn get_user_stats(&self, wallet_address: &str) -> Result<UserStats, ProgressError> {
        // Initialize stats if not exists
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO user_stats (wallet_address, last_activity_date)
            VALUES (?, ?)
            "#,
        )
        .bind(wallet_address)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        let total_courses_enrolled: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM user_progress WHERE wallet_address = ?")
                .bind(wallet_address)
                .fetch_one(&self.pool)
                .await?;

        let total_courses_completed: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_progress WHERE wallet_address = ? AND status = 'completed'",
        )
        .bind(wallet_address)
        .fetch_one(&self.pool)
        .await?;

        let total_lessons_completed: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM lesson_progress WHERE wallet_address = ? AND status = 'completed'"
        )
        .bind(wallet_address)
        .fetch_one(&self.pool)
        .await?;

        let total_quizzes_passed: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM quiz_attempts WHERE wallet_address = ? AND passed = 1",
        )
        .bind(wallet_address)
        .fetch_one(&self.pool)
        .await?;

        let total_challenges_completed: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM challenge_submissions WHERE wallet_address = ? AND status = 'approved'"
        )
        .bind(wallet_address)
        .fetch_one(&self.pool)
        .await?;

        let total_webinars_attended: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM webinar_attendance WHERE wallet_address = ?")
                .bind(wallet_address)
                .fetch_one(&self.pool)
                .await?;

        let total_mentor_sessions: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM mentor_sessions WHERE student_address = ? AND status = 'completed'"
        )
        .bind(wallet_address)
        .fetch_one(&self.pool)
        .await?;

        let row = sqlx::query("SELECT * FROM user_stats WHERE wallet_address = ?")
            .bind(wallet_address)
            .fetch_one(&self.pool)
            .await?;

        let total_xp: i64 = row.try_get("total_xp")?;
        let current_streak_days: i64 = row.try_get("current_streak_days")?;
        let longest_streak_days: i64 = row.try_get("longest_streak_days")?;
        let badges_json: String = row.try_get("badges_earned")?;
        let badges_earned: Vec<String> = serde_json::from_str(&badges_json)?;

        Ok(UserStats {
            wallet_address: wallet_address.to_string(),
            total_courses_enrolled,
            total_courses_completed,
            total_lessons_completed,
            total_quizzes_passed,
            total_challenges_completed,
            total_webinars_attended,
            total_mentor_sessions,
            total_xp,
            current_streak_days,
            longest_streak_days,
            badges_earned,
        })
    }

    pub async fn add_xp(&self, wallet_address: &str, xp: i64) -> Result<(), ProgressError> {
        sqlx::query(
            r#"
            INSERT INTO user_stats (wallet_address, total_xp, last_activity_date)
            VALUES (?, ?, ?)
            ON CONFLICT(wallet_address) DO UPDATE SET 
                total_xp = total_xp + excluded.total_xp
            "#,
        )
        .bind(wallet_address)
        .bind(xp)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn add_badge(
        &self,
        wallet_address: &str,
        badge_id: &str,
    ) -> Result<(), ProgressError> {
        let stats = self.get_user_stats(wallet_address).await?;
        let mut badges = stats.badges_earned;

        if !badges.contains(&badge_id.to_string()) {
            badges.push(badge_id.to_string());
        }

        let badges_json = serde_json::to_string(&badges)?;

        sqlx::query("UPDATE user_stats SET badges_earned = ? WHERE wallet_address = ?")
            .bind(badges_json)
            .bind(wallet_address)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn update_user_activity(&self, wallet_address: &str) -> Result<(), ProgressError> {
        let today = Utc::now().date_naive().to_string();

        // Get last activity date
        let last_activity: Option<String> = sqlx::query_scalar(
            "SELECT last_activity_date FROM user_stats WHERE wallet_address = ?",
        )
        .bind(wallet_address)
        .fetch_optional(&self.pool)
        .await?;

        let streak_increase = if let Some(last_date) = last_activity {
            let last = chrono::NaiveDate::parse_from_str(&last_date[..10], "%Y-%m-%d")
                .unwrap_or_else(|_| Utc::now().date_naive());
            let current = Utc::now().date_naive();
            let diff = (current - last).num_days();

            if diff == 1 {
                1 // Continue streak
            } else if diff > 1 {
                -999999 // Reset streak
            } else {
                0 // Same day
            }
        } else {
            1 // First activity
        };

        if streak_increase != 0 {
            if streak_increase > 0 {
                sqlx::query(
                    r#"
                    UPDATE user_stats 
                    SET current_streak_days = current_streak_days + ?,
                        longest_streak_days = MAX(longest_streak_days, current_streak_days + ?),
                        last_activity_date = ?
                    WHERE wallet_address = ?
                    "#,
                )
                .bind(streak_increase)
                .bind(streak_increase)
                .bind(today)
                .bind(wallet_address)
                .execute(&self.pool)
                .await?;
            } else {
                // Reset streak
                sqlx::query(
                    r#"
                    UPDATE user_stats 
                    SET current_streak_days = 1,
                        last_activity_date = ?
                    WHERE wallet_address = ?
                    "#,
                )
                .bind(today)
                .bind(wallet_address)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    pub async fn get_leaderboard(
        &self,
        limit: i64,
    ) -> Result<Vec<LeaderboardEntry>, ProgressError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                wallet_address,
                total_xp,
                current_streak_days,
                badges_earned
            FROM user_stats 
            ORDER BY total_xp DESC 
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut leaderboard = Vec::new();
        for (rank, row) in rows.iter().enumerate() {
            let wallet_address: String = row.try_get("wallet_address")?;
            let total_xp: i64 = row.try_get("total_xp")?;
            let streak_days: i64 = row.try_get("current_streak_days")?;
            let badges_json: String = row.try_get("badges_earned")?;
            let badges: Vec<String> = serde_json::from_str(&badges_json)?;

            let courses_completed: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM user_progress WHERE wallet_address = ? AND status = 'completed'"
            )
            .bind(&wallet_address)
            .fetch_one(&self.pool)
            .await?;

            leaderboard.push(LeaderboardEntry {
                wallet_address,
                rank: rank as i64 + 1,
                total_xp,
                courses_completed,
                badges_count: badges.len() as i64,
                streak_days,
            });
        }

        Ok(leaderboard)
    }

    // Helper methods
    fn user_progress_from_row(
        row: &sqlx::sqlite::SqliteRow,
    ) -> Result<UserProgress, ProgressError> {
        let status_str: String = row.try_get("status")?;
        let status = match status_str.as_str() {
            "notstarted" => ProgressStatus::NotStarted,
            "inprogress" => ProgressStatus::InProgress,
            "completed" => ProgressStatus::Completed,
            "failed" => ProgressStatus::Failed,
            _ => ProgressStatus::NotStarted,
        };

        let completed_json: String = row.try_get("completed_lessons")?;
        let completed_lessons: Vec<String> = serde_json::from_str(&completed_json)
            .map_err(|e| ProgressError::InvalidData(e.to_string()))?;

        let started_str: String = row.try_get("started_at")?;
        let last_accessed_str: String = row.try_get("last_accessed")?;
        let completed_str: Option<String> = row.try_get("completed_at")?;

        Ok(UserProgress {
            id: row.try_get("id")?,
            wallet_address: row.try_get("wallet_address")?,
            course_id: row.try_get("course_id")?,
            status,
            progress_percentage: row.try_get("progress_percentage")?,
            completed_lessons,
            quiz_attempts: row.try_get("quiz_attempts")?,
            last_accessed: DateTime::parse_from_rfc3339(&last_accessed_str)
                .map_err(|e| ProgressError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
            started_at: DateTime::parse_from_rfc3339(&started_str)
                .map_err(|e| ProgressError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
            completed_at: completed_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            }),
        })
    }

    fn lesson_progress_from_row(
        row: &sqlx::sqlite::SqliteRow,
    ) -> Result<LessonProgress, ProgressError> {
        let status_str: String = row.try_get("status")?;
        let status = match status_str.as_str() {
            "notstarted" => ProgressStatus::NotStarted,
            "inprogress" => ProgressStatus::InProgress,
            "completed" => ProgressStatus::Completed,
            "failed" => ProgressStatus::Failed,
            _ => ProgressStatus::NotStarted,
        };

        let started_str: String = row.try_get("started_at")?;
        let completed_str: Option<String> = row.try_get("completed_at")?;

        Ok(LessonProgress {
            id: row.try_get("id")?,
            wallet_address: row.try_get("wallet_address")?,
            lesson_id: row.try_get("lesson_id")?,
            course_id: row.try_get("course_id")?,
            status,
            time_spent_minutes: row.try_get("time_spent_minutes")?,
            completion_percentage: row.try_get("completion_percentage")?,
            last_position: row.try_get("last_position")?,
            started_at: DateTime::parse_from_rfc3339(&started_str)
                .map_err(|e| ProgressError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
            completed_at: completed_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            }),
        })
    }

    fn quiz_attempt_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<QuizAttempt, ProgressError> {
        let attempted_str: String = row.try_get("attempted_at")?;

        Ok(QuizAttempt {
            id: row.try_get("id")?,
            wallet_address: row.try_get("wallet_address")?,
            quiz_id: row.try_get("quiz_id")?,
            score: row.try_get("score")?,
            total_points: row.try_get("total_points")?,
            passed: row.try_get::<i64, _>("passed")? != 0,
            answers: row.try_get("answers")?,
            time_taken_minutes: row.try_get("time_taken_minutes")?,
            attempted_at: DateTime::parse_from_rfc3339(&attempted_str)
                .map_err(|e| ProgressError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
        })
    }

    fn challenge_submission_from_row(
        row: &sqlx::sqlite::SqliteRow,
    ) -> Result<ChallengeSubmission, ProgressError> {
        let submitted_str: String = row.try_get("submitted_at")?;
        let reviewed_str: Option<String> = row.try_get("reviewed_at")?;

        Ok(ChallengeSubmission {
            id: row.try_get("id")?,
            wallet_address: row.try_get("wallet_address")?,
            challenge_id: row.try_get("challenge_id")?,
            submission_data: row.try_get("submission_data")?,
            status: row.try_get("status")?,
            score: row.try_get("score")?,
            feedback: row.try_get("feedback")?,
            submitted_at: DateTime::parse_from_rfc3339(&submitted_str)
                .map_err(|e| ProgressError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
            reviewed_at: reviewed_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            }),
        })
    }

    fn mentor_session_from_row(
        row: &sqlx::sqlite::SqliteRow,
    ) -> Result<MentorSession, ProgressError> {
        let scheduled_str: String = row.try_get("scheduled_at")?;
        let created_str: String = row.try_get("created_at")?;
        let completed_str: Option<String> = row.try_get("completed_at")?;

        Ok(MentorSession {
            id: row.try_get("id")?,
            student_address: row.try_get("student_address")?,
            mentor_id: row.try_get("mentor_id")?,
            topic: row.try_get("topic")?,
            scheduled_at: DateTime::parse_from_rfc3339(&scheduled_str)
                .map_err(|e| ProgressError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
            duration_minutes: row.try_get("duration_minutes")?,
            status: row.try_get("status")?,
            notes: row.try_get("notes")?,
            student_rating: row.try_get("student_rating")?,
            mentor_rating: row.try_get("mentor_rating")?,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| ProgressError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
            completed_at: completed_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_status_serialization() {
        let status = ProgressStatus::InProgress;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"inprogress\"");
    }
}
