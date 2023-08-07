output "this" {
  value = aws_lambda_function.this
}

output "execution_role" {
  value = aws_iam_role.lambda_assume
}

output "log_group_name" {
  value = aws_cloudwatch_log_group.this.name
}
