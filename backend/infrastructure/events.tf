resource "aws_cloudwatch_event_rule" "every_ten_minutes" {
  name                = "${var.project_name}-${var.environment}-every-10-minutes"
  description         = "Runs every 10 minutes"
  schedule_expression = "rate(10 minutes)"
}
