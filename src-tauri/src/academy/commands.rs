use super::*;
use tauri::State;

// Course commands
#[tauri::command]
pub async fn create_course(
    academy: State<'_, SharedAcademyEngine>,
    course: content::Course,
) -> Result<content::Course, String> {
    academy
        .read()
        .await
        .content_service()
        .read()
        .await
        .create_course(course)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_course(
    academy: State<'_, SharedAcademyEngine>,
    id: String,
) -> Result<content::Course, String> {
    academy
        .read()
        .await
        .content_service()
        .read()
        .await
        .get_course(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_courses(
    academy: State<'_, SharedAcademyEngine>,
    category: Option<String>,
    level: Option<content::CourseLevel>,
) -> Result<Vec<content::Course>, String> {
    academy
        .read()
        .await
        .content_service()
        .read()
        .await
        .list_courses(category, level)
        .await
        .map_err(|e| e.to_string())
}

// Lesson commands
#[tauri::command]
pub async fn create_lesson(
    academy: State<'_, SharedAcademyEngine>,
    lesson: content::Lesson,
) -> Result<content::Lesson, String> {
    academy
        .read()
        .await
        .content_service()
        .read()
        .await
        .create_lesson(lesson)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_course_lessons(
    academy: State<'_, SharedAcademyEngine>,
    course_id: String,
) -> Result<Vec<content::Lesson>, String> {
    academy
        .read()
        .await
        .content_service()
        .read()
        .await
        .get_course_lessons(&course_id)
        .await
        .map_err(|e| e.to_string())
}

// Quiz commands
#[tauri::command]
pub async fn create_quiz(
    academy: State<'_, SharedAcademyEngine>,
    quiz: content::Quiz,
) -> Result<content::Quiz, String> {
    academy
        .read()
        .await
        .content_service()
        .read()
        .await
        .create_quiz(quiz)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_quiz(
    academy: State<'_, SharedAcademyEngine>,
    id: String,
) -> Result<content::Quiz, String> {
    academy
        .read()
        .await
        .content_service()
        .read()
        .await
        .get_quiz(&id)
        .await
        .map_err(|e| e.to_string())
}

// Challenge commands
#[tauri::command]
pub async fn create_challenge(
    academy: State<'_, SharedAcademyEngine>,
    challenge: content::Challenge,
) -> Result<content::Challenge, String> {
    academy
        .read()
        .await
        .content_service()
        .read()
        .await
        .create_challenge(challenge)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_challenges(
    academy: State<'_, SharedAcademyEngine>,
    active_only: bool,
) -> Result<Vec<content::Challenge>, String> {
    academy
        .read()
        .await
        .content_service()
        .read()
        .await
        .list_challenges(active_only)
        .await
        .map_err(|e| e.to_string())
}

// Webinar commands
#[tauri::command]
pub async fn create_webinar(
    academy: State<'_, SharedAcademyEngine>,
    webinar: content::Webinar,
) -> Result<content::Webinar, String> {
    academy
        .read()
        .await
        .content_service()
        .read()
        .await
        .create_webinar(webinar)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_webinars(
    academy: State<'_, SharedAcademyEngine>,
    status: Option<String>,
) -> Result<Vec<content::Webinar>, String> {
    academy
        .read()
        .await
        .content_service()
        .read()
        .await
        .list_webinars(status)
        .await
        .map_err(|e| e.to_string())
}

// Mentor commands
#[tauri::command]
pub async fn create_mentor(
    academy: State<'_, SharedAcademyEngine>,
    mentor: content::Mentor,
) -> Result<content::Mentor, String> {
    academy
        .read()
        .await
        .content_service()
        .read()
        .await
        .create_mentor(mentor)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_mentors(
    academy: State<'_, SharedAcademyEngine>,
    expertise_area: Option<String>,
) -> Result<Vec<content::Mentor>, String> {
    academy
        .read()
        .await
        .content_service()
        .read()
        .await
        .list_mentors(expertise_area)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_content_stats(
    academy: State<'_, SharedAcademyEngine>,
) -> Result<content::ContentStats, String> {
    academy
        .read()
        .await
        .content_service()
        .read()
        .await
        .get_stats()
        .await
        .map_err(|e| e.to_string())
}

// Progress commands
#[tauri::command]
pub async fn start_course(
    academy: State<'_, SharedAcademyEngine>,
    wallet_address: String,
    course_id: String,
) -> Result<progress::UserProgress, String> {
    academy
        .read()
        .await
        .progress_tracker()
        .read()
        .await
        .start_course(&wallet_address, &course_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_user_progress(
    academy: State<'_, SharedAcademyEngine>,
    wallet_address: String,
    course_id: String,
) -> Result<progress::UserProgress, String> {
    academy
        .read()
        .await
        .progress_tracker()
        .read()
        .await
        .get_user_progress(&wallet_address, &course_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn complete_course(
    academy: State<'_, SharedAcademyEngine>,
    wallet_address: String,
    course_id: String,
) -> Result<(), String> {
    let progress_tracker = academy.read().await.progress_tracker();

    progress_tracker
        .read()
        .await
        .complete_course(&wallet_address, &course_id)
        .await
        .map_err(|e| e.to_string())?;

    // Award course completion XP
    let content_service = academy.read().await.content_service();
    let course = content_service
        .read()
        .await
        .get_course(&course_id)
        .await
        .map_err(|e| e.to_string())?;

    let xp = course.xp_reward;
    progress_tracker
        .read()
        .await
        .add_xp(&wallet_address, xp)
        .await
        .map_err(|e| e.to_string())?;

    // Award badge if specified
    if let Some(badge_id) = course.badge_id {
        let reward_engine = academy.read().await.reward_engine();
        progress_tracker
            .read()
            .await
            .add_badge(&wallet_address, &badge_id)
            .await
            .map_err(|e| e.to_string())?;

        reward_engine
            .read()
            .await
            .award_badge(
                &wallet_address,
                &badge_id,
                &format!("Completed course: {}", course.title),
            )
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn start_lesson(
    academy: State<'_, SharedAcademyEngine>,
    wallet_address: String,
    lesson_id: String,
    course_id: String,
) -> Result<progress::LessonProgress, String> {
    academy
        .read()
        .await
        .progress_tracker()
        .read()
        .await
        .start_lesson(&wallet_address, &lesson_id, &course_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_lesson_progress(
    academy: State<'_, SharedAcademyEngine>,
    wallet_address: String,
    lesson_id: String,
) -> Result<progress::LessonProgress, String> {
    academy
        .read()
        .await
        .progress_tracker()
        .read()
        .await
        .get_lesson_progress(&wallet_address, &lesson_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_lesson_progress(
    academy: State<'_, SharedAcademyEngine>,
    wallet_address: String,
    lesson_id: String,
    time_spent: i64,
    last_position: Option<String>,
) -> Result<(), String> {
    academy
        .read()
        .await
        .progress_tracker()
        .read()
        .await
        .update_lesson_progress(&wallet_address, &lesson_id, time_spent, last_position)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn complete_lesson(
    academy: State<'_, SharedAcademyEngine>,
    wallet_address: String,
    lesson_id: String,
    course_id: String,
) -> Result<(), String> {
    let progress_tracker = academy.read().await.progress_tracker();

    progress_tracker
        .read()
        .await
        .complete_lesson(&wallet_address, &lesson_id, &course_id)
        .await
        .map_err(|e| e.to_string())?;

    // Award lesson XP
    let content_service = academy.read().await.content_service();
    let lessons = content_service
        .read()
        .await
        .get_course_lessons(&course_id)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(lesson) = lessons.iter().find(|l| l.id == lesson_id) {
        let xp = lesson.xp_reward;
        progress_tracker
            .read()
            .await
            .add_xp(&wallet_address, xp)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn submit_quiz(
    academy: State<'_, SharedAcademyEngine>,
    attempt: progress::QuizAttempt,
) -> Result<progress::QuizAttempt, String> {
    let progress_tracker = academy.read().await.progress_tracker();
    let result = progress_tracker
        .read()
        .await
        .submit_quiz(attempt.clone())
        .await
        .map_err(|e| e.to_string())?;

    // Award XP if passed
    if result.passed {
        let reward_engine = academy.read().await.reward_engine();
        let xp = reward_engine
            .read()
            .await
            .calculate_quiz_reward(
                result.score,
                result.total_points,
                result.time_taken_minutes < 30,
            )
            .await;

        progress_tracker
            .read()
            .await
            .add_xp(&result.wallet_address, xp)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(result)
}

#[tauri::command]
pub async fn get_quiz_attempts(
    academy: State<'_, SharedAcademyEngine>,
    wallet_address: String,
    quiz_id: String,
) -> Result<Vec<progress::QuizAttempt>, String> {
    academy
        .read()
        .await
        .progress_tracker()
        .read()
        .await
        .get_quiz_attempts(&wallet_address, &quiz_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn submit_challenge(
    academy: State<'_, SharedAcademyEngine>,
    submission: progress::ChallengeSubmission,
) -> Result<progress::ChallengeSubmission, String> {
    academy
        .read()
        .await
        .progress_tracker()
        .read()
        .await
        .submit_challenge(submission)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_challenge_submissions(
    academy: State<'_, SharedAcademyEngine>,
    wallet_address: String,
) -> Result<Vec<progress::ChallengeSubmission>, String> {
    academy
        .read()
        .await
        .progress_tracker()
        .read()
        .await
        .get_challenge_submissions(&wallet_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn record_webinar_attendance(
    academy: State<'_, SharedAcademyEngine>,
    attendance: progress::WebinarAttendance,
) -> Result<progress::WebinarAttendance, String> {
    let progress_tracker = academy.read().await.progress_tracker();
    let result = progress_tracker
        .read()
        .await
        .record_webinar_attendance(attendance.clone())
        .await
        .map_err(|e| e.to_string())?;

    // Award XP for attendance
    if result.duration_minutes >= 30 {
        let content_service = academy.read().await.content_service();
        let webinars = content_service
            .read()
            .await
            .list_webinars(None)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(webinar) = webinars.iter().find(|w| w.id == result.webinar_id) {
            let xp = webinar.xp_reward;
            progress_tracker
                .read()
                .await
                .add_xp(&result.wallet_address, xp)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn create_mentor_session(
    academy: State<'_, SharedAcademyEngine>,
    session: progress::MentorSession,
) -> Result<progress::MentorSession, String> {
    academy
        .read()
        .await
        .progress_tracker()
        .read()
        .await
        .create_mentor_session(session)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_user_mentor_sessions(
    academy: State<'_, SharedAcademyEngine>,
    wallet_address: String,
) -> Result<Vec<progress::MentorSession>, String> {
    academy
        .read()
        .await
        .progress_tracker()
        .read()
        .await
        .get_user_mentor_sessions(&wallet_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_user_stats(
    academy: State<'_, SharedAcademyEngine>,
    wallet_address: String,
) -> Result<progress::UserStats, String> {
    academy
        .read()
        .await
        .progress_tracker()
        .read()
        .await
        .get_user_stats(&wallet_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_leaderboard(
    academy: State<'_, SharedAcademyEngine>,
    limit: i64,
) -> Result<Vec<progress::LeaderboardEntry>, String> {
    academy
        .read()
        .await
        .progress_tracker()
        .read()
        .await
        .get_leaderboard(limit)
        .await
        .map_err(|e| e.to_string())
}

// Reward commands
#[tauri::command]
pub async fn create_badge(
    academy: State<'_, SharedAcademyEngine>,
    badge: rewards::Badge,
) -> Result<rewards::Badge, String> {
    academy
        .read()
        .await
        .reward_engine()
        .read()
        .await
        .create_badge(badge)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_badge(
    academy: State<'_, SharedAcademyEngine>,
    id: String,
) -> Result<rewards::Badge, String> {
    academy
        .read()
        .await
        .reward_engine()
        .read()
        .await
        .get_badge(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_badges(
    academy: State<'_, SharedAcademyEngine>,
) -> Result<Vec<rewards::Badge>, String> {
    academy
        .read()
        .await
        .reward_engine()
        .read()
        .await
        .list_badges()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn award_badge(
    academy: State<'_, SharedAcademyEngine>,
    wallet_address: String,
    badge_id: String,
    source: String,
) -> Result<rewards::EarnedBadge, String> {
    let reward_engine = academy.read().await.reward_engine();
    let progress_tracker = academy.read().await.progress_tracker();

    let earned_badge = reward_engine
        .read()
        .await
        .award_badge(&wallet_address, &badge_id, &source)
        .await
        .map_err(|e| e.to_string())?;

    progress_tracker
        .read()
        .await
        .add_badge(&wallet_address, &badge_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(earned_badge)
}

#[tauri::command]
pub async fn get_user_badges(
    academy: State<'_, SharedAcademyEngine>,
    wallet_address: String,
) -> Result<Vec<rewards::EarnedBadge>, String> {
    academy
        .read()
        .await
        .reward_engine()
        .read()
        .await
        .get_user_badges(&wallet_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn issue_certificate(
    academy: State<'_, SharedAcademyEngine>,
    certificate: rewards::Certificate,
) -> Result<rewards::Certificate, String> {
    academy
        .read()
        .await
        .reward_engine()
        .read()
        .await
        .issue_certificate(certificate)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_user_certificates(
    academy: State<'_, SharedAcademyEngine>,
    wallet_address: String,
) -> Result<Vec<rewards::Certificate>, String> {
    academy
        .read()
        .await
        .reward_engine()
        .read()
        .await
        .get_user_certificates(&wallet_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn verify_certificate(
    academy: State<'_, SharedAcademyEngine>,
    verification_code: String,
) -> Result<rewards::Certificate, String> {
    academy
        .read()
        .await
        .reward_engine()
        .read()
        .await
        .verify_certificate(&verification_code)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_user_rewards(
    academy: State<'_, SharedAcademyEngine>,
    wallet_address: String,
    unclaimed_only: bool,
) -> Result<Vec<rewards::Reward>, String> {
    academy
        .read()
        .await
        .reward_engine()
        .read()
        .await
        .get_user_rewards(&wallet_address, unclaimed_only)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn claim_reward(
    academy: State<'_, SharedAcademyEngine>,
    reward_id: String,
) -> Result<(), String> {
    academy
        .read()
        .await
        .reward_engine()
        .read()
        .await
        .claim_reward(&reward_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn claim_all_rewards(
    academy: State<'_, SharedAcademyEngine>,
    wallet_address: String,
) -> Result<i64, String> {
    academy
        .read()
        .await
        .reward_engine()
        .read()
        .await
        .claim_all_rewards(&wallet_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_reward_stats(
    academy: State<'_, SharedAcademyEngine>,
) -> Result<rewards::RewardStats, String> {
    academy
        .read()
        .await
        .reward_engine()
        .read()
        .await
        .get_reward_stats()
        .await
        .map_err(|e| e.to_string())
}
