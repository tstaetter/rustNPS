use crate::db::NpsEntry;
use crate::payloads::{NpsStats, TrendItem};
use bson::{DateTime, doc};
use chrono::{Datelike, Months, Utc};
use futures::TryStreamExt;
use mongodb::Collection;
use std::collections::HashMap;

type SegmentCounts = (u64, u64, u64);
type MonthSegmentData = HashMap<String, SegmentCounts>;

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

    let nps = calculate_nps(promoters, detractors, total);

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
    let now = Utc::now();

    // Calculate start of the month 5 months before current (6 months total)
    let five_months_ago = now.checked_sub_months(Months::new(5)).unwrap();
    let six_months_ago =
        chrono::NaiveDate::from_ymd_opt(five_months_ago.year(), five_months_ago.month(), 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap();

    let pipeline = vec![
        doc! { "$match": { "created_at": { "$gte": DateTime::from_chrono(six_months_ago) } } },
        doc! { "$addFields": {
            "year": { "$year": "$created_at" },
            "month": { "$month": "$created_at" }
        } },
        doc! { "$group": {
            "_id": { "year": "$year", "month": "$month", "segment": "$segment" },
            "total": { "$sum": 1 },
            "promoters": { "$sum": { "$cond": [{ "$gte": ["$score", 9] }, 1, 0] } },
            "detractors": { "$sum": { "$cond": [{ "$lte": ["$score", 6] }, 1, 0] } },
        } },
    ];

    // Collect aggregation results into a map keyed by (year, month) -> segment -> (total, promoters, detractors)
    let mut month_data: HashMap<(i32, i32), MonthSegmentData> = HashMap::new();

    if let Ok(mut cursor) = collection.aggregate(pipeline).await {
        while let Ok(Some(result)) = cursor.try_next().await {
            if let Ok(id) = result.get_document("_id") {
                let agg_year = id.get_i32("year").unwrap_or(0);
                let agg_month = id.get_i32("month").unwrap_or(0);
                let segment = id.get_str("segment").unwrap_or("").to_string();
                let total = result.get_i64("total").unwrap_or(0) as u64;
                let promoters = result.get_i64("promoters").unwrap_or(0) as u64;
                let detractors = result.get_i64("detractors").unwrap_or(0) as u64;

                month_data
                    .entry((agg_year, agg_month))
                    .or_default()
                    .insert(segment, (total, promoters, detractors));
            }
        }
    }

    // Build TrendItems for each of the last 6 months
    let mut trend = Vec::new();
    for i in (0..6).rev() {
        let date = now.checked_sub_months(Months::new(i)).unwrap();
        let label = date.format("%b %Y").to_string();
        let y = date.year();
        let m = date.month() as i32;

        let segments = month_data.get(&(y, m)).cloned().unwrap_or_default();

        let mut overall_total: u64 = 0;
        let mut overall_promoters: u64 = 0;
        let mut overall_detractors: u64 = 0;
        let mut by_segment = HashMap::new();

        for (segment, (total, promoters, detractors)) in &segments {
            overall_total += total;
            overall_promoters += promoters;
            overall_detractors += detractors;

            let nps = calculate_nps(*promoters, *detractors, *total);
            by_segment.insert(segment.clone(), nps);
        }

        let overall = calculate_nps(overall_promoters, overall_detractors, overall_total);

        trend.push(TrendItem {
            label,
            overall,
            by_segment,
            total: overall_total,
        });
    }

    trend
}

fn calculate_nps(promoters: u64, detractors: u64, total: u64) -> i32 {
    if total == 0 {
        0
    } else {
        let p_pct = (promoters as f64 / total as f64) * 100.0;
        let d_pct = (detractors as f64 / total as f64) * 100.0;
        (p_pct - d_pct).round() as i32
    }
}

fn percentage(count: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    ((count as f64 / total as f64) * 1000.0).round() / 10.0
}
