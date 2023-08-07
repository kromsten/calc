module "validate_xdefi_achievements_kujira_lambda" {
  source = "./modules/lambda"

  environment   = var.environment
  function_name = "validate-xdefi-achievements-${var.environment}-kujira"
  source_dir    = "../dist/validate-xdefi-achievements"
  timeout       = 60

  environment_variables = {
    CHAIN                = "kujira"
    DCA_CONTRACT_ADDRESS = "kujira1e6fjnq7q20sh9cca76wdkfg69esha5zn53jjewrtjgm4nktk824stzyysu"
    XDEFI_PARTNER_ID     = var.xdefi_partner_id
    NET_URL              = "https://rpc-kujira.mintthemoon.xyz"
    START_AFTER          = "2220"
  }
}

module "validate_xdefi_achievements_osmosis_lambda" {
  source = "./modules/lambda"

  environment   = var.environment
  function_name = "validate-xdefi-achievements-${var.environment}-osmosis"
  source_dir    = "../dist/validate-xdefi-achievements"
  timeout       = 60

  environment_variables = {
    CHAIN                = "osmosis"
    DCA_CONTRACT_ADDRESS = "osmo1zacxlu90sl6j2zf90uctpddhfmux84ryrw794ywnlcwx2zeh5a4q67qtc9"
    XDEFI_PARTNER_ID     = var.xdefi_partner_id
    NET_URL              = "https://rpc.osmosis.zone/"
    START_AFTER          = "1600"
  }
}
