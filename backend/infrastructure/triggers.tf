resource "aws_cloudwatch_event_target" "invoke_validate_xdefi_achievements_kujira" {
  rule = aws_cloudwatch_event_rule.every_ten_minutes.name
  arn  = module.validate_xdefi_achievements_kujira_lambda.this.arn

  depends_on = [
    aws_cloudwatch_event_rule.every_ten_minutes
  ]
}

resource "aws_cloudwatch_event_target" "invoke_validate_xdefi_achievements_osmosis" {
  rule = aws_cloudwatch_event_rule.every_ten_minutes.name
  arn  = module.validate_xdefi_achievements_osmosis_lambda.this.arn

  depends_on = [
    aws_cloudwatch_event_rule.every_ten_minutes
  ]
}
