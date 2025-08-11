use crate::models::quiz::{AnswerExportData, QuestionExportData};
use arrow_array::{
    ArrayRef, RecordBatch,
    builder::{
        BooleanBuilder, Float64Builder, Int32Builder, ListBuilder, StringBuilder, UInt64Builder,
    },
};
use arrow_schema::{DataType, Field, Fields, Schema};
use color_eyre::Result;
use parquet::arrow::ArrowWriter;
use std::sync::Arc;

/// Creates the Arrow schema that exactly matches what QuizAnalyticsBuilders generates
pub fn create_builder_compatible_schema() -> Schema {
    // Define the answer struct fields (all non-nullable as per builder)
    let answer_struct_fields = Fields::from(vec![
        Field::new("text", DataType::Utf8, false),
        Field::new("is_correct", DataType::Boolean, false),
        Field::new("sort_order", DataType::Int32, false),
        Field::new("selection_count", DataType::UInt64, false),
        Field::new("selection_percentage", DataType::Float64, false),
    ]);

    // Create the schema matching the builder output exactly
    Schema::new(vec![
        Field::new("question_id", DataType::Utf8, false),
        Field::new("question_text", DataType::Utf8, false),
        Field::new("explanation", DataType::Utf8, false),
        Field::new("difficulty_level", DataType::Utf8, false),
        Field::new("rule_references", DataType::Utf8, false),
        Field::new("total_attempts", DataType::UInt64, false),
        Field::new("correct_attempts", DataType::UInt64, false),
        Field::new("success_rate_percent", DataType::Float64, false),
        // The key difference: ListBuilder creates the inner "item" field with nullable: true
        Field::new(
            "answers",
            DataType::List(Arc::new(Field::new(
                "item",
                DataType::Struct(answer_struct_fields),
                true, // This is the critical part - ListBuilder makes this nullable: true
            ))),
            false,
        ),
        Field::new("created_at", DataType::Utf8, false),
        Field::new("updated_at", DataType::Utf8, false),
    ])
}

/// Builders for constructing Arrow arrays from quiz export data
pub struct QuizAnalyticsBuilders {
    question_id: StringBuilder,
    question_text: StringBuilder,
    explanation: StringBuilder,
    difficulty_level: StringBuilder,
    rule_references: StringBuilder,
    total_attempts: UInt64Builder,
    correct_attempts: UInt64Builder,
    success_rate_percent: Float64Builder,
    answers: ListBuilder<arrow_array::builder::StructBuilder>,
    created_at: StringBuilder,
    updated_at: StringBuilder,
}

impl QuizAnalyticsBuilders {
    pub fn new() -> Result<Self> {
        // Create answer struct fields exactly matching the schema
        let answer_struct_fields = Fields::from(vec![
            Field::new("text", DataType::Utf8, false),
            Field::new("is_correct", DataType::Boolean, false),
            Field::new("sort_order", DataType::Int32, false),
            Field::new("selection_count", DataType::UInt64, false),
            Field::new("selection_percentage", DataType::Float64, false),
        ]);

        // Create answer struct builder with the five fields
        let answer_struct_builder = arrow_array::builder::StructBuilder::new(
            answer_struct_fields,
            vec![
                Box::new(StringBuilder::new()) as Box<dyn arrow_array::builder::ArrayBuilder>,
                Box::new(BooleanBuilder::new()),
                Box::new(Int32Builder::new()),
                Box::new(UInt64Builder::new()),
                Box::new(Float64Builder::new()),
            ],
        );

        Ok(Self {
            question_id: StringBuilder::new(),
            question_text: StringBuilder::new(),
            explanation: StringBuilder::new(),
            difficulty_level: StringBuilder::new(),
            rule_references: StringBuilder::new(),
            total_attempts: UInt64Builder::new(),
            correct_attempts: UInt64Builder::new(),
            success_rate_percent: Float64Builder::new(),
            answers: ListBuilder::new(answer_struct_builder),
            created_at: StringBuilder::new(),
            updated_at: StringBuilder::new(),
        })
    }

    pub fn append_question(&mut self, question: &QuestionExportData) -> Result<()> {
        // Append basic question fields
        self.question_id.append_value(&question.question_id);
        self.question_text.append_value(&question.question_text);
        self.explanation.append_value(&question.explanation);
        self.difficulty_level
            .append_value(&question.difficulty_level);
        self.rule_references.append_value(&question.rule_references);
        self.total_attempts
            .append_value(question.total_attempts as u64);
        self.correct_attempts
            .append_value(question.correct_attempts as u64);
        self.success_rate_percent
            .append_value(question.success_rate_percent);
        self.created_at
            .append_value(question.created_at.format("%Y-%m-%d %H:%M:%S").to_string());
        self.updated_at
            .append_value(question.updated_at.format("%Y-%m-%d %H:%M:%S").to_string());

        // Append answers as nested List<Struct>
        for answer in &question.answers {
            self.append_answer(answer)?;
        }
        self.answers.append(true);

        Ok(())
    }

    fn append_answer(&mut self, answer: &AnswerExportData) -> Result<()> {
        let struct_builder = self.answers.values();

        // Append answer data to each field one by one to avoid multiple mutable borrows
        {
            let text_builder = struct_builder
                .field_builder::<StringBuilder>(0)
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get text builder"))?;
            text_builder.append_value(&answer.text);
        }

        {
            let is_correct_builder = struct_builder
                .field_builder::<BooleanBuilder>(1)
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get is_correct builder"))?;
            is_correct_builder.append_value(answer.is_correct);
        }

        {
            let sort_order_builder = struct_builder
                .field_builder::<Int32Builder>(2)
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get sort_order builder"))?;
            sort_order_builder.append_value(answer.sort_order);
        }

        {
            let selection_count_builder = struct_builder
                .field_builder::<UInt64Builder>(3)
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get selection_count builder"))?;
            selection_count_builder.append_value(answer.selection_count as u64);
        }

        {
            let selection_percentage_builder = struct_builder
                .field_builder::<Float64Builder>(4)
                .ok_or_else(|| {
                    color_eyre::eyre::eyre!("Failed to get selection_percentage builder")
                })?;
            selection_percentage_builder.append_value(answer.selection_percentage);
        }

        struct_builder.append(true);
        Ok(())
    }

    pub fn finish(mut self, schema: Arc<Schema>) -> Result<RecordBatch> {
        let columns: Vec<ArrayRef> = vec![
            Arc::new(self.question_id.finish()),
            Arc::new(self.question_text.finish()),
            Arc::new(self.explanation.finish()),
            Arc::new(self.difficulty_level.finish()),
            Arc::new(self.rule_references.finish()),
            Arc::new(self.total_attempts.finish()),
            Arc::new(self.correct_attempts.finish()),
            Arc::new(self.success_rate_percent.finish()),
            Arc::new(self.answers.finish()),
            Arc::new(self.created_at.finish()),
            Arc::new(self.updated_at.finish()),
        ];

        Ok(RecordBatch::try_new(schema, columns)?)
    }
}

/// Converts QuestionExportData to Arrow RecordBatch
pub fn questions_to_record_batch(questions: Vec<QuestionExportData>) -> Result<RecordBatch> {
    // Use the builder-compatible schema that matches exactly what builders generate
    let schema = Arc::new(create_builder_compatible_schema());
    let mut builders = QuizAnalyticsBuilders::new()?;

    for question in &questions {
        builders.append_question(question)?;
    }

    builders.finish(schema)
}

/// Writes RecordBatch to Parquet format
pub fn write_parquet(record_batch: RecordBatch) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    {
        let mut writer = ArrowWriter::try_new(&mut buffer, record_batch.schema(), None)?;
        writer.write(&record_batch)?;
        writer.close()?;
    }
    Ok(buffer)
}
