mod commands;
mod collector;
mod db;
mod help;
mod publisher;
mod utils;

use std::sync::Mutex;
use tauri::Manager;

/// 全局应用状态：SQLite 连接
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let conn = db::connection::init(&data_dir)
                .map_err(|e| format!("数据库初始化失败：{e}"))?;
            app.manage(AppState { db: Mutex::new(conn) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 文章
            commands::article_commands::list_articles,
            commands::article_commands::get_article,
            commands::article_commands::create_article,
            commands::article_commands::update_article,
            commands::article_commands::update_article_status,
            commands::article_commands::delete_articles,
            commands::article_commands::get_dashboard_stats,
            // 关键词任务
            commands::keyword_commands::list_keyword_tasks,
            commands::keyword_commands::create_keyword_task,
            commands::keyword_commands::update_keyword_task,
            commands::keyword_commands::delete_keyword_tasks,
            commands::keyword_commands::list_collect_logs,
            // 采集源
            commands::source_commands::list_sources,
            commands::source_commands::create_source,
            commands::source_commands::update_source,
            commands::source_commands::delete_sources,
            // 采集
            commands::collect_commands::collect_single_page,
            commands::collect_commands::run_keyword_task,
            commands::collect_commands::collect_from_source,
            commands::collect_commands::discover_from_sitemap,
            // 伪原创
            commands::rewrite_commands::get_rewrite_options,
            commands::rewrite_commands::save_rewrite_options,
            commands::rewrite_commands::preview_rewrite,
            commands::rewrite_commands::apply_rewrite,
            commands::rewrite_commands::import_custom_dict,
            commands::rewrite_commands::reset_custom_dict,
            commands::rewrite_commands::get_dict_stats,
            // 发布
            commands::publish_commands::list_cms_connections,
            commands::publish_commands::save_cms_connection,
            commands::publish_commands::delete_cms_connections,
            commands::publish_commands::test_cms_connection,
            commands::publish_commands::publish_article,
            commands::publish_commands::batch_publish,
            commands::publish_commands::export_hexo,
            commands::publish_commands::list_publish_records,
            // 设置
            commands::settings_commands::get_settings,
            commands::settings_commands::save_settings,
            commands::settings_commands::get_collect_settings,
            // 帮助系统
            commands::help_commands::get_manual_index,
            commands::help_commands::get_manual_section,
            commands::help_commands::get_examples,
            commands::help_commands::get_faq_categories,
            commands::help_commands::get_faqs,
            commands::help_commands::search_help,
            commands::help_commands::get_quick_start,
            commands::help_commands::get_changelog,
            commands::help_commands::get_shortcuts,
            commands::help_commands::get_error_codes,
            commands::help_commands::check_update,
            commands::help_commands::open_external_link,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
