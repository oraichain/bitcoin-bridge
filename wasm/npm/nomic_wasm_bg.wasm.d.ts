/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export function __wbg_depositaddress_free(a: number): void;
export function __wbg_get_depositaddress_sigsetIndex(a: number): number;
export function __wbg_set_depositaddress_sigsetIndex(a: number, b: number): void;
export function __wbg_validatorqueryinfo_free(a: number): void;
export function __wbg_get_validatorqueryinfo_jailed(a: number): number;
export function __wbg_set_validatorqueryinfo_jailed(a: number, b: number): void;
export function __wbg_get_validatorqueryinfo_commission(a: number, b: number): void;
export function __wbg_set_validatorqueryinfo_commission(a: number, b: number, c: number): void;
export function __wbg_get_validatorqueryinfo_inActiveSet(a: number): number;
export function __wbg_set_validatorqueryinfo_inActiveSet(a: number, b: number): void;
export function __wbg_get_validatorqueryinfo_info(a: number, b: number): void;
export function __wbg_set_validatorqueryinfo_info(a: number, b: number, c: number): void;
export function __wbg_delegation_free(a: number): void;
export function __wbg_get_delegation_address(a: number, b: number): void;
export function __wbg_set_delegation_address(a: number, b: number, c: number): void;
export function __wbg_get_delegation_liquid(a: number, b: number): void;
export function __wbg_set_delegation_liquid(a: number, b: number, c: number): void;
export function __wbg_get_delegation_unbonding(a: number, b: number): void;
export function __wbg_set_delegation_unbonding(a: number, b: number, c: number): void;
export function __wbg_coin_free(a: number): void;
export function __wbg_get_coin_denom(a: number): number;
export function __wbg_set_coin_denom(a: number, b: number): void;
export function __wbg_get_coin_amount(a: number): number;
export function __wbg_set_coin_amount(a: number, b: number): void;
export function __wbg_get_rewarddetails_claimed(a: number): number;
export function __wbg_set_rewarddetails_claimed(a: number, b: number): void;
export function __wbg_get_rewarddetails_claimable(a: number): number;
export function __wbg_set_rewarddetails_claimable(a: number, b: number): void;
export function __wbg_get_rewarddetails_amount(a: number): number;
export function __wbg_set_rewarddetails_amount(a: number, b: number): void;
export function __wbg_airdrop_free(a: number): void;
export function __wbg_get_airdrop_airdrop1(a: number): number;
export function __wbg_set_airdrop_airdrop1(a: number, b: number): void;
export function __wbg_get_airdrop_airdrop2(a: number): number;
export function __wbg_set_airdrop_airdrop2(a: number, b: number): void;
export function airdrop_total(a: number): number;
export function airdrop_claimedTotal(a: number): number;
export function main_js(): void;
export function __wbg_oraibtc_free(a: number): void;
export function oraibtc_new(a: number, b: number, c: number, d: number): number;
export function oraibtc_transfer(a: number, b: number, c: number, d: number): number;
export function oraibtc_balance(a: number, b: number, c: number): number;
export function oraibtc_nomRewardBalance(a: number, b: number, c: number): number;
export function oraibtc_nbtcRewardBalance(a: number, b: number, c: number): number;
export function oraibtc_delegations(a: number, b: number, c: number): number;
export function oraibtc_allValidators(a: number): number;
export function oraibtc_claim(a: number, b: number, c: number): number;
export function oraibtc_claimAirdrop1(a: number, b: number, c: number): number;
export function oraibtc_claimAirdrop2(a: number, b: number, c: number): number;
export function oraibtc_claimTestnetParticipationAirdrop(a: number, b: number, c: number): number;
export function oraibtc_claimTestnetParticipationIncentives(a: number, b: number, c: number): number;
export function oraibtc_claimIncomingIbcBtc(a: number, b: number, c: number): number;
export function oraibtc_setRecoveryAddress(a: number, b: number, c: number, d: number, e: number): number;
export function oraibtc_getRecoveryAddress(a: number, b: number, c: number): number;
export function oraibtc_delegate(a: number, b: number, c: number, d: number, e: number, f: number): number;
export function oraibtc_unbond(a: number, b: number, c: number, d: number, e: number, f: number): number;
export function oraibtc_redelegate(a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number): number;
export function oraibtc_airdropBalances(a: number, b: number, c: number): number;
export function oraibtc_incentiveBalances(a: number, b: number, c: number): number;
export function oraibtc_nonce(a: number, b: number, c: number): number;
export function oraibtc_generateDepositAddress(a: number, b: number, c: number): number;
export function oraibtc_nbtcBalance(a: number, b: number, c: number): number;
export function oraibtc_incomingIbcNbtcBalance(a: number, b: number, c: number): number;
export function oraibtc_valueLocked(a: number): number;
export function oraibtc_latestCheckpointHash(a: number): number;
export function oraibtc_bitcoinHeight(a: number): number;
export function oraibtc_capacityLimit(a: number): number;
export function oraibtc_depositsEnabled(a: number): number;
export function oraibtc_getAddress(a: number): number;
export function oraibtc_broadcastDepositAddress(a: number, b: number, c: number, d: number, e: number, f: number, g: number): number;
export function oraibtc_withdraw(a: number, b: number, c: number, d: number, e: number, f: number): number;
export function oraibtc_joinRewardAccounts(a: number, b: number, c: number, d: number, e: number): number;
export function oraibtc_ibcTransferOut(a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number): number;
export function oraibtc_convertEthAddress(a: number, b: number, c: number, d: number): void;
export function __wbg_jsiter_free(a: number): void;
export function jsiter_next(a: number, b: number): void;
export function __wbg_jsiternext_free(a: number): void;
export function __wbg_get_jsiternext_done(a: number): number;
export function __wbg_set_jsiternext_done(a: number, b: number): void;
export function jsiternext_value(a: number): number;
export function rustsecp256k1_v0_6_1_context_create(a: number): number;
export function rustsecp256k1_v0_6_1_context_destroy(a: number): void;
export function rustsecp256k1_v0_6_1_default_illegal_callback_fn(a: number, b: number): void;
export function rustsecp256k1_v0_6_1_default_error_callback_fn(a: number, b: number): void;
export function rustsecp256k1_v0_8_1_context_create(a: number): number;
export function rustsecp256k1_v0_8_1_context_destroy(a: number): void;
export function rustsecp256k1_v0_8_1_default_illegal_callback_fn(a: number, b: number): void;
export function rustsecp256k1_v0_8_1_default_error_callback_fn(a: number, b: number): void;
export function __wbg_set_validatorqueryinfo_amountStaked(a: number, b: number): void;
export function __wbg_set_unbondinfo_startSeconds(a: number, b: number): void;
export function __wbg_set_depositaddress_expiration(a: number, b: number): void;
export function __wbg_set_delegation_staked(a: number, b: number): void;
export function __wbg_set_rewarddetails_locked(a: number, b: number): void;
export function __wbg_set_unbondinfo_amount(a: number, b: number): void;
export function __wbg_set_validatorqueryinfo_address(a: number, b: number, c: number): void;
export function __wbg_set_depositaddress_address(a: number, b: number, c: number): void;
export function __wbg_get_validatorqueryinfo_amountStaked(a: number): number;
export function __wbg_get_unbondinfo_startSeconds(a: number): number;
export function __wbg_get_depositaddress_expiration(a: number): number;
export function __wbg_get_delegation_staked(a: number): number;
export function __wbg_get_rewarddetails_locked(a: number): number;
export function __wbg_get_unbondinfo_amount(a: number): number;
export function incentives_total(a: number): number;
export function incentives_claimedTotal(a: number): number;
export function __wbg_get_validatorqueryinfo_address(a: number, b: number): void;
export function __wbg_get_depositaddress_address(a: number, b: number): void;
export function __wbg_get_incentives_testnetParticipation(a: number): number;
export function __wbg_set_incentives_testnetParticipation(a: number, b: number): void;
export function __wbg_unbondinfo_free(a: number): void;
export function __wbg_rewarddetails_free(a: number): void;
export function __wbg_incentives_free(a: number): void;
export function __wbindgen_malloc(a: number, b: number): number;
export function __wbindgen_realloc(a: number, b: number, c: number, d: number): number;
export const __wbindgen_export_2: WebAssembly.Table;
export function wasm_bindgen__convert__closures__invoke1_mut__h5f79931d848bcbe7(a: number, b: number, c: number): void;
export function __wbindgen_free(a: number, b: number, c: number): void;
export function __wbindgen_exn_store(a: number): void;
export function wasm_bindgen__convert__closures__invoke2_mut__h755fea732ceba39a(a: number, b: number, c: number, d: number): void;
export function __wbindgen_add_to_stack_pointer(a: number): number;
export function __wbindgen_start(): void;
