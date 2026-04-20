use crate::db::NpsEntry;
use crate::payloads::{NpsStats, TrendItem};
use bson::{doc, DateTime};
use chrono::{Datelike, Duration, Utc};
use futures::TryStreamExt;
use mongodb::Collection;
use std::collections::HashMap;

pub(crate) async fn build_stats(
    collection: &Collection<NpsEntry>,
    filter: bson::Document,
) -> NpsStats {
    let total = collection
        .count_documents(filter.clone())
        .await
        .unwrap_or(0);

    let mut promoter_filter = filter.clone();
    promoter_filter.insert("score", doc! { "$gte": 9 });
    let promoters = collection
        .count_documents(promoter_filter)
        .await
        .unwrap_or(0);

    let mut passive_filter = filter.clone();
    passive_filter.insert("score", doc! { "$gte": 7, "$lte": 8 });
    let passives = collection
        .count_documents(passive_filter)
        .await
        .unwrap_or(0);

    let mut detractor_filter = filter.clone();
    detractor_filter.insert("score", doc! { "$lte": 6 });
    let detractors = collection
        .count_documents(detractor_filter)
        .await
        .unwrap_or(0);

    let nps = if total == 0 {
        0
    } else {
        let p_pct = (promoters as f64 / total as f64) * 100.0;
        let d_pct = (detractors as f64 / total as f64) * 100.0;
        (p_pct - d_pct).round() as i32
    };

    let average = if total == 0 {
        0.0
    } else {
        let pipeline = vec![
            doc! { "$match": filter },
            doc! { "$group": { "_id": null, "avg": { "$avg": "$score" } } },
        ];
        let mut cursor = collection.aggregate(pipeline).await.unwrap();
        if let Some(result) = cursor.try_next().await.unwrap() {
            (result.get_f64("avg").unwrap_or(0.0) * 100.0).round() / 100.0
        } else {
            0.0
        }
    };

    NpsStats {
        total,
        promoters,
        passives,
        detractors,
        nps,
        promoter_pct: percentage(promoters, total),
        passive_pct: percentage(passives, total),
        detractor_pct: percentage(detractors, total),
        average,
    }
}

pub(crate) async fn build_trend(collection: &Collection<NpsEntry>) -> Vec<TrendItem> {
    let mut trend = Vec::new();
    let now = Utc::now();

    for i in (0..6).rev() {
        let i = i as i32;
        // Actually we need to subtract i months
        let mut year = now.year();
        let mut month = now.month() as i32 - i;
        while month <= 0 {
            month += 12;
            year -= 1;
        }
        let start_of_month = chrono::NaiveDate::from_ymd_opt(year, month as u32, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap();

        let next_month = if month == 12 {
            chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            chrono::NaiveDate::from_ymd_opt(year, month as u32 + 1, 1).unwrap()
        }
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Utc)
        .unwrap();

        let end_of_month = next_month - Duration::nanoseconds(1);

        let filter = doc! {
            "created_at": {
                "$gte": DateTime::from_chrono(start_of_month),
                "$lte": DateTime::from_chrono(end_of_month)
            }
        };

        let total = collection
            .count_documents(filter.clone())
            .await
            .unwrap_or(0);
        let overall_nps = calculate_nps(collection, filter.clone()).await;

        let segments_names: Vec<String> = collection
            .distinct("segment", filter.clone())
            .await
            .unwrap_or_default()
            .into_iter()
            .filter_map(|b| b.as_str().map(|s| s.to_string()))
            .collect();

        let mut by_segment = HashMap::new();
        for s in segments_names {
            let mut s_filter = filter.clone();
            s_filter.insert("segment", &s);
            by_segment.insert(s, calculate_nps(collection, s_filter).await);
        }

        trend.push(TrendItem {
            label: start_of_month.format("%b %Y").to_string(),
            overall: overall_nps,
            by_segment,
            total,
        });
    }

    trend
}

async fn calculate_nps(collection: &Collection<NpsEntry>, filter: bson::Document) -> i32 {
    let total = collection
        .count_documents(filter.clone())
        .await
        .unwrap_or(0);
    if total == 0 {
        return 0;
    }

    let mut promoter_filter = filter.clone();
    promoter_filter.insert("score", doc! { "$gte": 9 });
    let promoters = collection
        .count_documents(promoter_filter)
        .await
        .unwrap_or(0);

    let mut detractor_filter = filter.clone();
    detractor_filter.insert("score", doc! { "$lte": 6 });
    let detractors = collection
        .count_documents(detractor_filter)
        .await
        .unwrap_or(0);

    let p_pct = (promoters as f64 / total as f64) * 100.0;
    let d_pct = (detractors as f64 / total as f64) * 100.0;
    (p_pct - d_pct).round() as i32
}

fn percentage(count: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    ((count as f64 / total as f64) * 1000.0).round() / 10.0
}
