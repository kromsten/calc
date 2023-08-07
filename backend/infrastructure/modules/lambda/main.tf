data "archive_file" "handler_zip" {
  count = var.source_dir != null ? 1 : 0
  type  = "zip"

  source_dir  = var.source_dir
  output_path = "${path.module}/${basename(var.source_dir)}.zip"
}

resource "aws_lambda_function" "this" {
  filename                       = var.source_dir != null ? data.archive_file.handler_zip[0].output_path : null
  function_name                  = var.function_name
  handler                        = var.handler_function != null ? "app.${var.handler_function}" : null
  source_code_hash               = var.source_code_hash != null ? var.source_code_hash : var.source_dir != null ? filebase64sha256(data.archive_file.handler_zip[0].output_path) : null
  role                           = aws_iam_role.lambda_assume.arn
  runtime                        = var.runtime != null ? var.runtime : null
  memory_size                    = var.memory_size
  timeout                        = var.timeout
  publish                        = true
  reserved_concurrent_executions = var.reserved_concurrent_executions
  image_uri                      = var.image_uri != null ? var.image_uri : null
  package_type                   = var.package_type

  environment {
    variables = var.environment_variables
  }

  depends_on = [
    aws_iam_role.lambda_assume,
    aws_iam_role_policy.lambda_allow,
    aws_cloudwatch_log_group.this,
  ]
}

resource "aws_cloudwatch_log_group" "this" {
  name              = "/aws/lambda/${var.function_name}"
  retention_in_days = var.environment == "prod" ? 365 : 5
}

data "aws_iam_policy_document" "lambda_assume" {
  statement {
    effect = "Allow"
    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
    actions = ["sts:AssumeRole"]
  }
}

resource "aws_iam_role" "lambda_assume" {
  name               = var.function_name
  assume_role_policy = data.aws_iam_policy_document.lambda_assume.json
}

data "aws_iam_policy_document" "lambda_allow" {
  statement {
    effect    = "Allow"
    resources = ["arn:aws:logs:*:*:*"]
    actions = [
      "logs:CreateLogGroup",
      "logs:CreateLogStream",
      "logs:PutLogEvents",
      "logs:PutMetricFilter",
      "logs:PutRetentionPolicy"
    ]
  }
}

resource "aws_iam_role_policy" "lambda_allow" {
  name   = "${var.function_name}-allow-logs"
  role   = aws_iam_role.lambda_assume.id
  policy = data.aws_iam_policy_document.lambda_allow.json
}
