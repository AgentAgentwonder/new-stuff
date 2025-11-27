use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const ACADEMY_DB_FILE: &str = "academy.db";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CourseLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Video,
    Tutorial,
    Article,
    Quiz,
    Challenge,
    Webinar,
    LiveSession,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    pub id: String,
    pub title: String,
    pub description: String,
    pub level: CourseLevel,
    pub category: String,
    pub duration_minutes: i64,
    pub xp_reward: i64,
    pub badge_id: Option<String>,
    pub prerequisites: Vec<String>,
    pub tags: Vec<String>,
    pub thumbnail_url: Option<String>,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lesson {
    pub id: String,
    pub course_id: String,
    pub title: String,
    pub description: String,
    pub content_type: ContentType,
    pub content_url: Option<String>,
    pub content_data: Option<String>, // JSON for interactive content
    pub order_index: i64,
    pub duration_minutes: i64,
    pub xp_reward: i64,
    pub is_mandatory: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quiz {
    pub id: String,
    pub lesson_id: String,
    pub title: String,
    pub questions: Vec<QuizQuestion>,
    pub passing_score: i64,
    pub max_attempts: Option<i64>,
    pub time_limit_minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizQuestion {
    pub id: String,
    pub question: String,
    pub options: Vec<String>,
    pub correct_answer: usize,
    pub explanation: String,
    pub points: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Challenge {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub difficulty: CourseLevel,
    pub xp_reward: i64,
    pub badge_id: Option<String>,
    pub requirements: String,        // JSON
    pub validation_criteria: String, // JSON
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Webinar {
    pub id: String,
    pub title: String,
    pub description: String,
    pub instructor: String,
    pub scheduled_at: DateTime<Utc>,
    pub duration_minutes: i64,
    pub max_participants: Option<i64>,
    pub meeting_url: Option<String>,
    pub recording_url: Option<String>,
    pub xp_reward: i64,
    pub status: String, // scheduled, live, completed, cancelled
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mentor {
    pub id: String,
    pub wallet_address: String,
    pub name: String,
    pub bio: String,
    pub expertise_areas: Vec<String>,
    pub languages: Vec<String>,
    pub availability: String, // JSON schedule
    pub rating: f64,
    pub total_sessions: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentStats {
    pub total_courses: i64,
    pub total_lessons: i64,
    pub total_challenges: i64,
    pub total_webinars: i64,
    pub total_mentors: i64,
    pub active_students: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum ContentError {
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

pub struct ContentService {
    pool: Pool<Sqlite>,
}

impl ContentService {
    pub async fn new(app_handle: &AppHandle) -> Result<Self, ContentError> {
        let app_dir = app_handle.path().app_data_dir().map_err(|err| {
            ContentError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Unable to resolve app data directory: {err}"),
            ))
        })?;

        let _ = std::fs::create_dir_all(&app_dir);
        let db_path = app_dir.join(ACADEMY_DB_FILE);
        let db_url = format!("sqlite:{}", db_path.display());

        let pool = match SqlitePool::connect(&db_url).await {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!("Warning: ContentService failed to connect to {:?}: {}", db_path, e);
                eprintln!("Falling back to in-memory database for ContentService");
                eprintln!("ContentService using in-memory database for this session");
                SqlitePool::connect("sqlite::memory:").await?
            }
        };

        // Initialize database schema
        Self::init_schema(&pool).await?;

        Ok(Self { pool })
    }

    async fn init_schema(pool: &Pool<Sqlite>) -> Result<(), ContentError> {
        // Courses table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS courses (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                level TEXT NOT NULL,
                category TEXT NOT NULL,
                duration_minutes INTEGER NOT NULL,
                xp_reward INTEGER NOT NULL,
                badge_id TEXT,
                prerequisites TEXT, -- JSON array
                tags TEXT, -- JSON array
                thumbnail_url TEXT,
                is_published INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Lessons table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS lessons (
                id TEXT PRIMARY KEY NOT NULL,
                course_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                content_type TEXT NOT NULL,
                content_url TEXT,
                content_data TEXT, -- JSON
                order_index INTEGER NOT NULL,
                duration_minutes INTEGER NOT NULL,
                xp_reward INTEGER NOT NULL,
                is_mandatory INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (course_id) REFERENCES courses(id)
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Quizzes table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS quizzes (
                id TEXT PRIMARY KEY NOT NULL,
                lesson_id TEXT NOT NULL,
                title TEXT NOT NULL,
                questions TEXT NOT NULL, -- JSON
                passing_score INTEGER NOT NULL,
                max_attempts INTEGER,
                time_limit_minutes INTEGER,
                FOREIGN KEY (lesson_id) REFERENCES lessons(id)
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Challenges table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS challenges (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                category TEXT NOT NULL,
                difficulty TEXT NOT NULL,
                xp_reward INTEGER NOT NULL,
                badge_id TEXT,
                requirements TEXT NOT NULL, -- JSON
                validation_criteria TEXT NOT NULL, -- JSON
                start_date TEXT,
                end_date TEXT,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Webinars table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS webinars (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                instructor TEXT NOT NULL,
                scheduled_at TEXT NOT NULL,
                duration_minutes INTEGER NOT NULL,
                max_participants INTEGER,
                meeting_url TEXT,
                recording_url TEXT,
                xp_reward INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'scheduled',
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Mentors table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS mentors (
                id TEXT PRIMARY KEY NOT NULL,
                wallet_address TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                bio TEXT NOT NULL,
                expertise_areas TEXT NOT NULL, -- JSON array
                languages TEXT NOT NULL, -- JSON array
                availability TEXT NOT NULL, -- JSON
                rating REAL NOT NULL DEFAULT 5.0,
                total_sessions INTEGER NOT NULL DEFAULT 0,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_courses_category ON courses(category)")
            .execute(pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_courses_level ON courses(level)")
            .execute(pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_lessons_course ON lessons(course_id)")
            .execute(pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_mentors_wallet ON mentors(wallet_address)")
            .execute(pool)
            .await?;

        Ok(())
    }

    // Course operations
    pub async fn create_course(&self, course: Course) -> Result<Course, ContentError> {
        let prerequisites_json = serde_json::to_string(&course.prerequisites)?;
        let tags_json = serde_json::to_string(&course.tags)?;
        let level_str = format!("{:?}", course.level).to_lowercase();

        sqlx::query(
            r#"
            INSERT INTO courses (
                id, title, description, level, category, duration_minutes,
                xp_reward, badge_id, prerequisites, tags, thumbnail_url,
                is_published, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&course.id)
        .bind(&course.title)
        .bind(&course.description)
        .bind(level_str)
        .bind(&course.category)
        .bind(course.duration_minutes)
        .bind(course.xp_reward)
        .bind(&course.badge_id)
        .bind(prerequisites_json)
        .bind(tags_json)
        .bind(&course.thumbnail_url)
        .bind(course.is_published)
        .bind(course.created_at.to_rfc3339())
        .bind(course.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(course)
    }

    pub async fn get_course(&self, id: &str) -> Result<Course, ContentError> {
        let row = sqlx::query("SELECT * FROM courses WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| ContentError::NotFound(format!("Course not found: {}", id)))?;

        Self::course_from_row(&row)
    }

    pub async fn list_courses(
        &self,
        category: Option<String>,
        level: Option<CourseLevel>,
    ) -> Result<Vec<Course>, ContentError> {
        let mut query = "SELECT * FROM courses WHERE is_published = 1".to_string();

        if category.is_some() {
            query.push_str(" AND category = ?");
        }
        if level.is_some() {
            query.push_str(" AND level = ?");
        }
        query.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::query(&query);

        if let Some(cat) = category {
            q = q.bind(cat);
        }
        if let Some(lvl) = level {
            let level_str = format!("{:?}", lvl).to_lowercase();
            q = q.bind(level_str);
        }

        let rows = q.fetch_all(&self.pool).await?;

        let mut courses = Vec::new();
        for row in rows {
            courses.push(Self::course_from_row(&row)?);
        }

        Ok(courses)
    }

    // Lesson operations
    pub async fn create_lesson(&self, lesson: Lesson) -> Result<Lesson, ContentError> {
        let content_type_str = format!("{:?}", lesson.content_type).to_lowercase();

        sqlx::query(
            r#"
            INSERT INTO lessons (
                id, course_id, title, description, content_type, content_url,
                content_data, order_index, duration_minutes, xp_reward,
                is_mandatory, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&lesson.id)
        .bind(&lesson.course_id)
        .bind(&lesson.title)
        .bind(&lesson.description)
        .bind(content_type_str)
        .bind(&lesson.content_url)
        .bind(&lesson.content_data)
        .bind(lesson.order_index)
        .bind(lesson.duration_minutes)
        .bind(lesson.xp_reward)
        .bind(lesson.is_mandatory)
        .bind(lesson.created_at.to_rfc3339())
        .bind(lesson.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(lesson)
    }

    pub async fn get_course_lessons(&self, course_id: &str) -> Result<Vec<Lesson>, ContentError> {
        let rows =
            sqlx::query("SELECT * FROM lessons WHERE course_id = ? ORDER BY order_index ASC")
                .bind(course_id)
                .fetch_all(&self.pool)
                .await?;

        let mut lessons = Vec::new();
        for row in rows {
            lessons.push(Self::lesson_from_row(&row)?);
        }

        Ok(lessons)
    }

    // Quiz operations
    pub async fn create_quiz(&self, quiz: Quiz) -> Result<Quiz, ContentError> {
        let questions_json = serde_json::to_string(&quiz.questions)?;

        sqlx::query(
            r#"
            INSERT INTO quizzes (
                id, lesson_id, title, questions, passing_score,
                max_attempts, time_limit_minutes
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&quiz.id)
        .bind(&quiz.lesson_id)
        .bind(&quiz.title)
        .bind(questions_json)
        .bind(quiz.passing_score)
        .bind(quiz.max_attempts)
        .bind(quiz.time_limit_minutes)
        .execute(&self.pool)
        .await?;

        Ok(quiz)
    }

    pub async fn get_quiz(&self, id: &str) -> Result<Quiz, ContentError> {
        let row = sqlx::query("SELECT * FROM quizzes WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| ContentError::NotFound(format!("Quiz not found: {}", id)))?;

        Self::quiz_from_row(&row)
    }

    // Challenge operations
    pub async fn create_challenge(&self, challenge: Challenge) -> Result<Challenge, ContentError> {
        let difficulty_str = format!("{:?}", challenge.difficulty).to_lowercase();

        sqlx::query(
            r#"
            INSERT INTO challenges (
                id, title, description, category, difficulty, xp_reward,
                badge_id, requirements, validation_criteria, start_date,
                end_date, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&challenge.id)
        .bind(&challenge.title)
        .bind(&challenge.description)
        .bind(&challenge.category)
        .bind(difficulty_str)
        .bind(challenge.xp_reward)
        .bind(&challenge.badge_id)
        .bind(&challenge.requirements)
        .bind(&challenge.validation_criteria)
        .bind(challenge.start_date.map(|d| d.to_rfc3339()))
        .bind(challenge.end_date.map(|d| d.to_rfc3339()))
        .bind(challenge.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(challenge)
    }

    pub async fn list_challenges(&self, active_only: bool) -> Result<Vec<Challenge>, ContentError> {
        let now = Utc::now().to_rfc3339();

        let query = if active_only {
            format!("SELECT * FROM challenges WHERE (start_date IS NULL OR start_date <= '{}') AND (end_date IS NULL OR end_date >= '{}') ORDER BY created_at DESC", now, now)
        } else {
            "SELECT * FROM challenges ORDER BY created_at DESC".to_string()
        };

        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        let mut challenges = Vec::new();
        for row in rows {
            challenges.push(Self::challenge_from_row(&row)?);
        }

        Ok(challenges)
    }

    // Webinar operations
    pub async fn create_webinar(&self, webinar: Webinar) -> Result<Webinar, ContentError> {
        sqlx::query(
            r#"
            INSERT INTO webinars (
                id, title, description, instructor, scheduled_at, duration_minutes,
                max_participants, meeting_url, recording_url, xp_reward, status, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&webinar.id)
        .bind(&webinar.title)
        .bind(&webinar.description)
        .bind(&webinar.instructor)
        .bind(webinar.scheduled_at.to_rfc3339())
        .bind(webinar.duration_minutes)
        .bind(webinar.max_participants)
        .bind(&webinar.meeting_url)
        .bind(&webinar.recording_url)
        .bind(webinar.xp_reward)
        .bind(&webinar.status)
        .bind(webinar.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(webinar)
    }

    pub async fn list_webinars(
        &self,
        status: Option<String>,
    ) -> Result<Vec<Webinar>, ContentError> {
        let query = if let Some(s) = status {
            format!(
                "SELECT * FROM webinars WHERE status = '{}' ORDER BY scheduled_at ASC",
                s
            )
        } else {
            "SELECT * FROM webinars ORDER BY scheduled_at ASC".to_string()
        };

        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        let mut webinars = Vec::new();
        for row in rows {
            webinars.push(Self::webinar_from_row(&row)?);
        }

        Ok(webinars)
    }

    // Mentor operations
    pub async fn create_mentor(&self, mentor: Mentor) -> Result<Mentor, ContentError> {
        let expertise_json = serde_json::to_string(&mentor.expertise_areas)?;
        let languages_json = serde_json::to_string(&mentor.languages)?;

        sqlx::query(
            r#"
            INSERT INTO mentors (
                id, wallet_address, name, bio, expertise_areas, languages,
                availability, rating, total_sessions, is_active, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&mentor.id)
        .bind(&mentor.wallet_address)
        .bind(&mentor.name)
        .bind(&mentor.bio)
        .bind(expertise_json)
        .bind(languages_json)
        .bind(&mentor.availability)
        .bind(mentor.rating)
        .bind(mentor.total_sessions)
        .bind(mentor.is_active)
        .bind(mentor.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(mentor)
    }

    pub async fn list_mentors(
        &self,
        expertise_area: Option<String>,
    ) -> Result<Vec<Mentor>, ContentError> {
        let rows = sqlx::query("SELECT * FROM mentors WHERE is_active = 1 ORDER BY rating DESC")
            .fetch_all(&self.pool)
            .await?;

        let mut mentors = Vec::new();
        for row in rows {
            let mentor = Self::mentor_from_row(&row)?;
            if let Some(area) = &expertise_area {
                if mentor.expertise_areas.contains(area) {
                    mentors.push(mentor);
                }
            } else {
                mentors.push(mentor);
            }
        }

        Ok(mentors)
    }

    pub async fn get_stats(&self) -> Result<ContentStats, ContentError> {
        let total_courses: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM courses WHERE is_published = 1")
                .fetch_one(&self.pool)
                .await?;

        let total_lessons: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM lessons")
            .fetch_one(&self.pool)
            .await?;

        let total_challenges: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM challenges")
            .fetch_one(&self.pool)
            .await?;

        let total_webinars: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM webinars")
            .fetch_one(&self.pool)
            .await?;

        let total_mentors: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM mentors WHERE is_active = 1")
                .fetch_one(&self.pool)
                .await?;

        Ok(ContentStats {
            total_courses,
            total_lessons,
            total_challenges,
            total_webinars,
            total_mentors,
            active_students: 0, // Will be calculated from progress tracker
        })
    }

    // Helper methods to convert from database rows
    fn course_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Course, ContentError> {
        let level_str: String = row.try_get("level")?;
        let level = match level_str.as_str() {
            "beginner" => CourseLevel::Beginner,
            "intermediate" => CourseLevel::Intermediate,
            "advanced" => CourseLevel::Advanced,
            "expert" => CourseLevel::Expert,
            _ => CourseLevel::Beginner,
        };

        let prerequisites_json: String = row.try_get("prerequisites")?;
        let prerequisites: Vec<String> = serde_json::from_str(&prerequisites_json)?;

        let tags_json: String = row.try_get("tags")?;
        let tags: Vec<String> = serde_json::from_str(&tags_json)?;

        let created_str: String = row.try_get("created_at")?;
        let updated_str: String = row.try_get("updated_at")?;

        Ok(Course {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            level,
            category: row.try_get("category")?,
            duration_minutes: row.try_get("duration_minutes")?,
            xp_reward: row.try_get("xp_reward")?,
            badge_id: row.try_get("badge_id")?,
            prerequisites,
            tags,
            thumbnail_url: row.try_get("thumbnail_url")?,
            is_published: row.try_get::<i64, _>("is_published")? != 0,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| ContentError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map_err(|e| ContentError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
        })
    }

    fn lesson_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Lesson, ContentError> {
        let content_type_str: String = row.try_get("content_type")?;
        let content_type = match content_type_str.as_str() {
            "video" => ContentType::Video,
            "tutorial" => ContentType::Tutorial,
            "article" => ContentType::Article,
            "quiz" => ContentType::Quiz,
            "challenge" => ContentType::Challenge,
            "webinar" => ContentType::Webinar,
            "livesession" => ContentType::LiveSession,
            _ => ContentType::Tutorial,
        };

        let created_str: String = row.try_get("created_at")?;
        let updated_str: String = row.try_get("updated_at")?;

        Ok(Lesson {
            id: row.try_get("id")?,
            course_id: row.try_get("course_id")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            content_type,
            content_url: row.try_get("content_url")?,
            content_data: row.try_get("content_data")?,
            order_index: row.try_get("order_index")?,
            duration_minutes: row.try_get("duration_minutes")?,
            xp_reward: row.try_get("xp_reward")?,
            is_mandatory: row.try_get::<i64, _>("is_mandatory")? != 0,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| ContentError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map_err(|e| ContentError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
        })
    }

    fn quiz_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Quiz, ContentError> {
        let questions_json: String = row.try_get("questions")?;
        let questions: Vec<QuizQuestion> = serde_json::from_str(&questions_json)?;

        Ok(Quiz {
            id: row.try_get("id")?,
            lesson_id: row.try_get("lesson_id")?,
            title: row.try_get("title")?,
            questions,
            passing_score: row.try_get("passing_score")?,
            max_attempts: row.try_get("max_attempts")?,
            time_limit_minutes: row.try_get("time_limit_minutes")?,
        })
    }

    fn challenge_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Challenge, ContentError> {
        let difficulty_str: String = row.try_get("difficulty")?;
        let difficulty = match difficulty_str.as_str() {
            "beginner" => CourseLevel::Beginner,
            "intermediate" => CourseLevel::Intermediate,
            "advanced" => CourseLevel::Advanced,
            "expert" => CourseLevel::Expert,
            _ => CourseLevel::Beginner,
        };

        let created_str: String = row.try_get("created_at")?;
        let start_date_str: Option<String> = row.try_get("start_date")?;
        let end_date_str: Option<String> = row.try_get("end_date")?;

        Ok(Challenge {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            category: row.try_get("category")?,
            difficulty,
            xp_reward: row.try_get("xp_reward")?,
            badge_id: row.try_get("badge_id")?,
            requirements: row.try_get("requirements")?,
            validation_criteria: row.try_get("validation_criteria")?,
            start_date: start_date_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            }),
            end_date: end_date_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            }),
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| ContentError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
        })
    }

    fn webinar_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Webinar, ContentError> {
        let scheduled_str: String = row.try_get("scheduled_at")?;
        let created_str: String = row.try_get("created_at")?;

        Ok(Webinar {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            instructor: row.try_get("instructor")?,
            scheduled_at: DateTime::parse_from_rfc3339(&scheduled_str)
                .map_err(|e| ContentError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
            duration_minutes: row.try_get("duration_minutes")?,
            max_participants: row.try_get("max_participants")?,
            meeting_url: row.try_get("meeting_url")?,
            recording_url: row.try_get("recording_url")?,
            xp_reward: row.try_get("xp_reward")?,
            status: row.try_get("status")?,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| ContentError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
        })
    }

    fn mentor_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Mentor, ContentError> {
        let expertise_json: String = row.try_get("expertise_areas")?;
        let expertise_areas: Vec<String> = serde_json::from_str(&expertise_json)?;

        let languages_json: String = row.try_get("languages")?;
        let languages: Vec<String> = serde_json::from_str(&languages_json)?;

        let created_str: String = row.try_get("created_at")?;

        Ok(Mentor {
            id: row.try_get("id")?,
            wallet_address: row.try_get("wallet_address")?,
            name: row.try_get("name")?,
            bio: row.try_get("bio")?,
            expertise_areas,
            languages,
            availability: row.try_get("availability")?,
            rating: row.try_get("rating")?,
            total_sessions: row.try_get("total_sessions")?,
            is_active: row.try_get::<i64, _>("is_active")? != 0,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| ContentError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_course_level_serialization() {
        let level = CourseLevel::Beginner;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"beginner\"");
    }

    #[test]
    fn test_content_type_serialization() {
        let content = ContentType::Video;
        let json = serde_json::to_string(&content).unwrap();
        assert_eq!(json, "\"video\"");
    }
}
