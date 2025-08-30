//! Bilibili API 测试文件
//!
//! 这个文件测试所有实现的 Bilibili API 方法

use music_plugin_sdk::types::{SearchQuery, SearchType, PageInput};
use music_plugin_sdk::traits::media::MediaPlugin;
use music_plugin_sdk::traits::MediaAuthPlugin;
use std::collections::HashMap;
use crate::internal::bilibili::plugin::BilibiliPlugin;

#[tokio::test]
async fn test_all_apis() {
    println!("=== 开始测试 Bilibili API 实现 ===");
    
    // 创建插件实例
    let plugin = BilibiliPlugin::new();
    println!("✓ 插件实例创建成功");
    
    // 测试 is_authenticated 方法
    println!("\n--- 测试 is_authenticated 方法 ---");
    let auth_status = plugin.is_authenticated();
    println!("认证状态: {}", auth_status);
    println!("✓ is_authenticated 方法测试完成");
    
    // 测试 search 方法
    println!("\n--- 测试 search 方法 ---");
    let search_query = SearchQuery {
        query: "旅人".to_string(),
        types: vec![SearchType::Track],
        page: Some(PageInput {
            limit: Some(5),
            offset: Some(0),
            cursor: None,
        }),
        per_type_page: None,
        sort: None,
        per_type_sort: None,
        filters: HashMap::new(),
        provider_params: HashMap::new(),
    };
    
    match plugin.search(&search_query).await {
        Ok(result) => {
            println!("✓ 搜索成功");
            println!("提供商: {}", result.provider);
            println!("找到音轨数量: {}", result.tracks.items.len());
            
            if !result.tracks.items.is_empty() {
                println!("前3个搜索结果:");
                for (i, track) in result.tracks.items.iter().take(3).enumerate() {
                    println!("  {}. {} - {}", i + 1, track.title, track.artist);
                    println!("     ID: {}", track.id);
                    println!("     时长: {:?}秒", track.duration);
                    println!("     播放量: {:?}", track.popularity);
                    if let Some(ref urls) = track.external_urls {
                        if let Some(bilibili_url) = urls.get("bilibili" as &str) {
                            println!("     URL: {}", bilibili_url);
                        }
                    }
                    println!();
                }
                
                // 使用第一个音轨进行后续测试
                let first_track = &result.tracks.items[0];
                
                // 测试 get_track 方法
                println!("\n--- 测试 get_track 方法 ---");
                match plugin.get_track(&first_track.id).await {
                    Ok(track) => {
                        println!("✓ 获取音轨详情成功");
                        println!("标题: {}", track.title);
                        println!("艺术家: {}", track.artist);
                        println!("时长: {:?}秒", track.duration);
                        println!("封面URL: {:?}", track.cover_url);
                        println!("元数据: {:?}", track.metadata);
                        
                        // 测试字幕功能
                        println!("\n--- 测试字幕功能 ---");
                        if let Some(ref lyrics) = track.lyrics {
                            println!("✓ 字幕获取成功");
                            println!("字幕语言: {:?}", lyrics.language);
                            println!("字幕格式: {:?}", lyrics.format);
                            println!("字幕是否同步: {}", lyrics.synced);
                            println!("字幕来源: {:?}", lyrics.source);
                            println!("字幕文本预览: {}", &lyrics.text[..lyrics.text.len().min(100)]);
                            
                            if let Some(ref translations) = lyrics.translations {
                                println!("字幕翻译数量: {}", translations.len());
                                if !translations.is_empty() {
                                    let first_translation = &translations[0];
                                    println!("字幕语言: {}", first_translation.language);
                                    println!("字幕是否同步: {}", first_translation.synced);
                                    println!("字幕格式: {:?}", first_translation.format);
                                    println!("前3行字幕:");
                                    for (i, line) in first_translation.lines.iter().take(3).enumerate() {
                                        println!("  {}. [{}ms] {}", i + 1, line.timestamp_ms.unwrap_or(0), line.text);
                                    }
                                }
                            }
                        } else {
                            println!("✗ 该音轨没有可用的字幕");
                        }
                    }
                    Err(e) => {
                        println!("✗ 获取音轨详情失败: {:?}", e);
                    }
                }
                
                // 测试 is_track_available 方法
                println!("\n--- 测试 is_track_available 方法 ---");
                match plugin.is_track_available(&first_track.id).await {
                    Ok(available) => {
                        println!("✓ 检查音轨可用性成功");
                        println!("音轨 {} 可用: {}", first_track.id, available);
                    }
                    Err(e) => {
                        println!("✗ 检查音轨可用性失败: {:?}", e);
                    }
                }
                
                // 测试 get_media_stream 方法
                println!("\n--- 测试 get_media_stream 方法 ---");
                match plugin.get_media_stream(&first_track.id).await {
                    Ok(url) => {
                        println!("✓ 获取流URL成功");
                        println!("流URL: {}", url);
                    }
                    Err(e) => {
                        println!("✗ 获取流URL失败: {:?}", e);
                    }
                }
                
                // 测试 get_artist 方法
                if let Some(ref artists) = first_track.artists {
                    if !artists.is_empty() {
                        let artist_id = &artists[0].id;
                        println!("\n--- 测试 get_artist 方法 ---");
                        match plugin.get_artist(artist_id).await {
                            Ok(artist) => {
                                println!("✓ 获取艺术家信息成功");
                                println!("艺术家名称: {}", artist.name);
                                println!("描述: {:?}", artist.description);
                                println!("头像URL: {:?}", artist.avatar_url);
                                println!("元数据: {:?}", artist.metadata);
                            }
                            Err(e) => {
                                println!("✗ 获取艺术家信息失败: {:?}", e);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("✗ 搜索失败: {:?}", e);
        }
    }
    
    // 测试 get_album 方法
    println!("\n--- 测试 get_album 方法 ---");
    match plugin.get_album("test_album_id").await {
        Ok(album) => {
            println!("✓ 获取专辑成功: {:?}", album);
        }
        Err(e) => {
            println!("✓ 获取专辑失败（预期）: {:?}", e);
        }
    }
    
    // 测试 get_playlist 方法
    println!("\n--- 测试 get_playlist 方法 ---");
    match plugin.get_playlist("123456").await {
        Ok(playlist) => {
            println!("✓ 获取播放列表成功");
            println!("标题: {}", playlist.title);
            println!("描述: {:?}", playlist.description);
            println!("音轨数量: {:?}", playlist.total_tracks);
            println!("创建者: {}", playlist.creator);
            println!("封面URL: {:?}", playlist.cover_url);
            println!("是否公开: {}", playlist.is_public);
        }
        Err(e) => {
            println!("✗ 获取播放列表失败: {:?}", e);
        }
    }
    
    // 测试 get_user_playlists 方法
    println!("\n--- 测试 get_user_playlists 方法 ---");
    match plugin.get_user_playlists().await {
        Ok(playlists) => {
            println!("✓ 获取用户播放列表成功");
            println!("播放列表数量: {}", playlists.len());
            for (i, playlist) in playlists.iter().take(3).enumerate() {
                println!("  {}. {} ({} 音轨)", i + 1, playlist.title, playlist.total_tracks.unwrap_or(0));
            }
        }
        Err(e) => {
            println!("✗ 获取用户播放列表失败: {:?}", e);
        }
    }
    
    println!("\n=== 所有 API 测试完成 ===");
}