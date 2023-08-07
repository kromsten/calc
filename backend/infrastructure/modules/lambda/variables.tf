variable "environment" {
  description = "Deployment environment (dev, test, staging, prod)"
  type        = string
  default     = "dev"
}

variable "function_name" {
  description = "Lambda function name"
  type        = string
}

variable "source_dir" {
  description = "Path to the directory containing the handler code"
  type        = string
  default     = null
}

variable "handler_function" {
  description = "Name of the handler function"
  type        = string
  default     = "handler"
}

variable "environment_variables" {
  type = map(string)
  default = {
    placeholder = "empty_value"
  }
}

variable "runtime" {
  description = "Identifier of the function's runtime"
  type        = string
  default     = "nodejs16.x"
}

variable "memory_size" {
  type    = number
  default = 512
}

variable "timeout" {
  type    = number
  default = 15
}

variable "reserved_concurrent_executions" {
  type    = number
  default = -1
}

variable "image_uri" {
  type    = string
  default = null
}

variable "package_type" {
  type    = string
  default = "Zip"
}

variable "source_code_hash" {
  type    = string
  default = null
}
