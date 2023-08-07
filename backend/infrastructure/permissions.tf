resource "aws_lambda_permission" "allow_cloudwatch_to_invoke_validate_xdefi_achievements_kujira" {
  statement_id  = "AllowExecutionFromCloudWatch"
  action        = "lambda:InvokeFunction"
  function_name = module.validate_xdefi_achievements_kujira_lambda.this.function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.every_ten_minutes.arn
}

resource "aws_lambda_permission" "allow_cloudwatch_to_invoke_validate_xdefi_achievements_osmosis" {
  statement_id  = "AllowExecutionFromCloudWatch"
  action        = "lambda:InvokeFunction"
  function_name = module.validate_xdefi_achievements_osmosis_lambda.this.function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.every_ten_minutes.arn
}
