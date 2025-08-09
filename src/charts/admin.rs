use super::{ChartGenerator, ChartTheme};
use crate::models::{
    AnswerDistribution, DailyAttemptsByDifficulty, DifficultyPerformance, QuestionStatistics,
};
use charming::{
    component::{Axis, Grid, Legend, Title},
    element::{AxisType, NameLocation, Tooltip, Trigger},
    series::{Bar, Line, Pie},
};
use color_eyre::eyre::Result;

/// Admin chart implementations
pub struct AdminCharts;

impl AdminCharts {
    /// Generate stacked area chart showing daily attempts by difficulty with success/fail breakdown
    pub fn daily_attempts_by_difficulty(
        daily_data: Vec<DailyAttemptsByDifficulty>,
    ) -> Result<String> {
        // Handle empty data case
        if daily_data.is_empty() {
            let chart = ChartTheme::base_config()
                .title(
                    Title::new()
                        .text("Daily Quiz Attempts by Difficulty - No Data Available")
                        .left("center"),
                )
                .series(Line::new().name("No Data").data(vec![0.0]));
            return ChartGenerator::generate_svg(chart);
        }

        // Extract dates for x-axis
        let dates: Vec<String> = daily_data.iter().map(|d| d.date.clone()).collect();

        // Collect all unique difficulties across all days
        let mut all_difficulties: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for day in &daily_data {
            for difficulty in day.difficulty_attempts.keys() {
                all_difficulties.insert(difficulty.clone());
            }
        }
        let mut difficulties: Vec<String> = all_difficulties.into_iter().collect();
        difficulties.sort(); // Ensure consistent ordering

        // Create series data for each difficulty + outcome combination
        let mut series_list = Vec::new();

        for difficulty in &difficulties {
            // Success series for this difficulty
            let success_data: Vec<i32> = daily_data
                .iter()
                .map(|day| {
                    day.difficulty_attempts
                        .get(difficulty)
                        .map(|breakdown| breakdown.success_count as i32)
                        .unwrap_or(0)
                })
                .collect();

            // Fail series for this difficulty
            let fail_data: Vec<i32> = daily_data
                .iter()
                .map(|day| {
                    day.difficulty_attempts
                        .get(difficulty)
                        .map(|breakdown| breakdown.fail_count as i32)
                        .unwrap_or(0)
                })
                .collect();

            // Add success series (stacked)
            series_list.push(
                Line::new()
                    .name(format!("{} Success", difficulty.to_uppercase()))
                    .stack(format!("{}_stack", difficulty)) // Stack by difficulty
                    .area_style(Default::default()) // Make it an area chart
                    .data(success_data),
            );

            // Add fail series (stacked on top of success)
            series_list.push(
                Line::new()
                    .name(format!("{} Fail", difficulty.to_uppercase()))
                    .stack(format!("{}_stack", difficulty)) // Same stack as success
                    .area_style(Default::default()) // Make it an area chart
                    .data(fail_data),
            );
        }

        let chart = ChartTheme::base_config()
            .title(
                Title::new()
                    .text("Daily Quiz Attempts by Difficulty Level")
                    .left("center"),
            )
            .tooltip(Tooltip::new().trigger(Trigger::Axis))
            .legend(Legend::new().top("40px").left("center"))
            .x_axis(
                Axis::new()
                    .type_(AxisType::Category)
                    .data(dates)
                    .name("Date")
                    .name_location(NameLocation::Center)
                    .name_gap(40),
            )
            .y_axis(
                Axis::new()
                    .type_(AxisType::Value)
                    .name("Number of Attempts")
                    .name_location(NameLocation::Center)
                    .name_gap(60)
                    .min(0),
            )
            .grid(
                Grid::new()
                    .left("15%")
                    .right("10%")
                    .top("25%")
                    .bottom("20%"),
            );

        // Add all series to the chart
        let mut final_chart = chart;
        for series in series_list {
            final_chart = final_chart.series(series);
        }

        ChartGenerator::generate_svg(final_chart)
    }

    /// Generate difficulty distribution horizontal bar chart
    pub fn difficulty_distribution(performance: Vec<DifficultyPerformance>) -> Result<String> {
        // Handle empty data case
        if performance.is_empty() {
            let chart = ChartTheme::base_config()
                .title(
                    Title::new()
                        .text("Question Distribution by Difficulty - No Data Available")
                        .left("center"),
                )
                .series(Bar::new().name("Question Count").data(vec![0]));
            return ChartGenerator::generate_svg(chart);
        }

        let difficulties: Vec<String> = performance.iter().map(|p| p.difficulty.clone()).collect();
        let question_counts: Vec<i32> = performance
            .iter()
            .map(|p| p.question_count as i32)
            .collect();

        let chart = ChartTheme::base_config()
            .title(
                Title::new()
                    .text("Question Distribution by Difficulty")
                    .left("center"),
            )
            .tooltip(Tooltip::new().trigger(Trigger::Axis))
            .x_axis(
                Axis::new()
                    .type_(AxisType::Value)
                    .name("Question Count")
                    .name_location(NameLocation::Center)
                    .name_gap(40),
            )
            .y_axis(
                Axis::new()
                    .type_(AxisType::Category)
                    .data(difficulties)
                    .name("Difficulty Level")
                    .name_location(NameLocation::Center)
                    .name_gap(80),
            )
            .grid(
                Grid::new()
                    .left("20%")
                    .right("10%")
                    .top("15%")
                    .bottom("20%"),
            )
            .series(Bar::new().name("Question Count").data(question_counts));

        ChartGenerator::generate_svg(chart)
    }

    /// Generate question performance horizontal bar chart (worst performing questions)
    pub fn question_performance(
        questions: Vec<QuestionStatistics>,
        limit: usize,
    ) -> Result<String> {
        // Handle empty data case
        if questions.is_empty() {
            let chart = ChartTheme::base_config()
                .title(
                    Title::new()
                        .text("Questions Needing Attention - No Data Available")
                        .left("center"),
                )
                .series(Bar::new().name("Success Rate").data(vec![0.0]));
            return ChartGenerator::generate_svg(chart);
        }

        // Take worst performing questions (lowest success rates)
        let mut sorted_questions = questions;
        sorted_questions.sort_by(|a, b| {
            a.success_rate
                .partial_cmp(&b.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let limited_questions: Vec<_> = sorted_questions.into_iter().take(limit).collect();

        // Truncate long question texts for display
        let question_labels: Vec<String> = limited_questions
            .iter()
            .map(|q| {
                if q.question_text.len() > 50 {
                    format!("{}...", &q.question_text[..47])
                } else {
                    q.question_text.clone()
                }
            })
            .collect();

        let success_rates: Vec<f64> = limited_questions.iter().map(|q| q.success_rate).collect();

        let chart = ChartTheme::base_config()
            .title(
                Title::new()
                    .text("Questions Needing Attention (Lowest Success Rates)")
                    .left("center"),
            )
            .tooltip(Tooltip::new().trigger(Trigger::Axis))
            .x_axis(
                Axis::new()
                    .type_(AxisType::Value)
                    .name("Success Rate (%)")
                    .name_location(NameLocation::Center)
                    .name_gap(40)
                    .min(0)
                    .max(100),
            )
            .y_axis(
                Axis::new()
                    .type_(AxisType::Category)
                    .data(question_labels)
                    .name("Questions")
                    .name_location(NameLocation::Center)
                    .name_gap(180),
            )
            .grid(
                Grid::new()
                    .left("45%")
                    .right("10%")
                    .top("15%")
                    .bottom("20%"),
            )
            .series(Bar::new().name("Success Rate").data(success_rates));

        ChartGenerator::generate_svg(chart)
    }

    /// Generate answer distribution pie chart for a specific question
    pub fn answer_distribution(
        question_text: &str,
        distribution: Vec<AnswerDistribution>,
    ) -> Result<String> {
        // Handle empty data case
        if distribution.is_empty() {
            let chart = ChartTheme::base_config()
                .title(
                    Title::new()
                        .text(format!(
                            "Answer Distribution: {} - No Data Available",
                            if question_text.len() > 60 {
                                format!("{}...", &question_text[..57])
                            } else {
                                question_text.to_string()
                            }
                        ))
                        .left("center"),
                )
                .series(
                    Pie::new()
                        .name("Answer Selection")
                        .data(vec![(1.0, "No data")]),
                );
            return ChartGenerator::generate_svg(chart);
        }

        // Convert distribution to pie chart data (value, name) format
        let pie_data: Vec<(f64, String)> = distribution
            .into_iter()
            .map(|d| {
                let label = if d.answer_text.len() > 30 {
                    format!("{}...", &d.answer_text[..27])
                } else {
                    d.answer_text.clone()
                };
                (
                    d.selection_count as f64,
                    format!("{} ({:.1}%)", label, d.selection_percentage),
                )
            })
            .collect();

        let chart = ChartTheme::base_config()
            .title(
                Title::new()
                    .text(format!(
                        "Answer Distribution: {}",
                        if question_text.len() > 60 {
                            format!("{}...", &question_text[..57])
                        } else {
                            question_text.to_string()
                        }
                    ))
                    .left("center"),
            )
            .tooltip(
                Tooltip::new()
                    .trigger(Trigger::Item)
                    .formatter("{b}: {c} selections ({d}%)"),
            )
            .legend(Legend::new().bottom("0%").left("center"))
            .series(
                Pie::new()
                    .name("Answer Selection")
                    .radius(vec!["0%", "70%"])
                    .center(vec!["50%", "45%"])
                    .data(pie_data),
            );

        ChartGenerator::generate_svg(chart)
    }
}

