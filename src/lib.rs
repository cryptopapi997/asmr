#![allow(unexpected_cfgs)]
#![allow(unused)] /* jesus christ */
#![cfg_attr(target_os = "solana", feature(asm_experimental_arch, asm_const))]

use std::mem::MaybeUninit;

use pinocchio::{
    account_info::AccountInfo, log::sol_log_64, msg, pubkey::Pubkey, syscalls::sol_log_pubkey,
};

/// IF YOU USE THIS PLS REMEMBER TO USE THIS AS HEAPSTART
///
/// YOU WILL HAVE TO REWRITE ALLOCATOR
const fn heap_start(num_accounts: usize) -> usize {
    0x300000000 + num_accounts * 8
}

const ACCOUNT_INFO_SIZE: usize = 88;
const MAX_PERMITTED_ACCOUNT_DATA_SIZE: usize = 10240;
const RENT_EPOCH_SIZE: usize = 8;
const TOTAL_ACCOUNT_DATA_TO_SKIP: usize =
    ACCOUNT_INFO_SIZE + MAX_PERMITTED_ACCOUNT_DATA_SIZE + RENT_EPOCH_SIZE + 7;
const ACCOUNTS_PTR: usize = 0x300000000;

#[no_mangle]
pub unsafe extern "C" fn entrypoint(mut input: *mut u8) -> u32 {
    let mut num_accounts = MaybeUninit::<usize>::uninit();
    #[cfg(target_os = "solana")]
    core::arch::asm!(
        // Load num accounts (r9 will be used to move into num_accounts stack..)
        "ldxdw r9, [r1 + 0]",
        "jeq r9, 0, 1006f", // check there's at least one account
        // Initialize accounts cursor
        "lddw r7, {accounts_ptr}",

        // first account is GUARANTEED to be nondup so we don't need dup check
        // inline nondup case
        /* START INLINE */
        // Store account ptr and load account data len
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        // Advance input cursor by static data, account data, and round up to next 8
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        /* END INLINE (DON'T NEED JUMP) */

        // We inline everything from 1 to 64 accounts (technically would need up to 128, but that makes my 
        // editor lag & it's the same idea) anyway. The structure is if we have e.g. 10 accounts, we jump to 
        // 93f - the 9 case, as we already processed the first account. Each marker is the count followed by 3 for 
        // the chunk that decides if we're duplicate or not, and 1 for the case we are and 0 for the case we're not.
        // We reach this by doing a fucked jump table via binary search, this can be made way better by using an actual
        // jump table, but I couldn't get it to work. Another optimization would be to put all the <count>3 cases at the end,
        // as those get hit only once per run. Right now they're in the middle between the dupe and non-dupe cases, which means
        // we need an extra jump to jump over those each time instead of being able to fall through in one of the 
        //dupe/non-dupe cases.

        /* BEGIN BINARY SEARCH "JUMP TABLE" */

        // Binary search jump table, max value is 64 so we start at 32
        "jeq r9, 32, 313f",
        "jgt r9, 32, 32001f",

        // Smaller than 32 and greater than 0
        "32000:",
        "jeq r9, 16, 153f",
        "jgt r9, 16, 16001f",

        // Smaller than 16 and greater than 0
        "16000:",
        "jeq r9, 8, 73f",
        "jgt r9, 8, 8001f",

        // Smaller than 8 and greater than 0
        "8000:",
        "jeq r9, 4, 33f",
        "jgt r9, 4, 4001f",

        // Smaller than 4 and greater than 0
        "4000:",
        "jeq r9, 2, 13f",
        // Has to be 3 otherwise
        "ja 23f",

        // Smaller than 2 and greater than 0
        "2000:",
        // Jump straight to 1 case
        "ja 93f",        

        // Greater than 4 and smaller than 8
        "4001:",
        "jeq r9, 6, 53f",
        "jgt r9, 6, 6001f",

        // Smaller than 6 and greater than 4
        "6000:",
        "jeq r9, 5, 43f",
        "ja 53f",

        // Larger than 6 and smaller than 8
        "6001:",
        "jeq r9, 7, 63f",
        "ja 73f",

        // Greater than 8 and smaller than 16
        "8001:",
        "jeq r9, 12, 113f",
        "jgt r9, 12, 12001f",

        // Smaller than 12 and greater than 8
        "12000:",
        "jeq r9, 10, 93f",
        "jgt r9, 10, 10001f",

        // Smaller than 10 and greater than 8
        "10000:",
        "jeq r9, 9, 83f",
        "ja 93f",

        // Larger than 10 and smaller than 12
        "10001:",
        "jeq r9, 11, 103f",
        "ja 113f",

        // Greater than 12 and smaller than 16
        "12001:",
        "jeq r9, 14, 133f",
        "jgt r9, 14, 14001f",

        // Smaller than 14 and greater than 12
        "14000:",
        "jeq r9, 13, 123f",
        "ja 133f",

        // Larger than 14 and smaller than 16
        "14001:",
        "jeq r9, 15, 143f",
        "ja 153f",

        // Greater than 16 and smaller than 32
        "16001:",
        "jeq r9, 24, 233f",
        "jgt r9, 24, 24001f",

        // Smaller than 24 and greater than 16
        "24000:",
        "jeq r9, 20, 193f",
        "jgt r9, 20, 20001f",

        // Smaller than 20 and greater than 16
        "20000:",
        "jeq r9, 18, 173f",
        "jgt r9, 18, 18001f",

        // Smaller than 18 and greater than 16
        "18000:",
        "jeq r9, 17, 163f",
        "ja 173f",

        // Larger than 18 and smaller than 20
        "18001:",
        "jeq r9, 19, 183f",
        "ja 193f",

        // Greater than 20 and smaller than 24
        "20001:",
        "jeq r9, 22, 213f",
        "jgt r9, 22, 22001f",

        // Smaller than 22 and greater than 20
        "22000:",
        "jeq r9, 21, 203f",
        "ja 213f",

        // Larger than 22 and smaller than 24
        "22001:",
        "jeq r9, 23, 223f",
        "ja 233f",

        // Greater than 24 and smaller than 32
        "24001:",
        "jeq r9, 28, 273f",
        "jgt r9, 28, 28001f",

        // Smaller than 28 and greater than 24
        "28000:",
        "jeq r9, 26, 253f",
        "jgt r9, 26, 26001f",

        // Smaller than 26 and greater than 24
        "26000:",
        "jeq r9, 25, 243f",
        "ja 253f",

        // Larger than 26 and smaller than 28
        "26001:",
        "jeq r9, 27, 263f",
        "ja 273f",

        // Greater than 28 and smaller than 32
        "28001:",
        "jeq r9, 30, 293f",
        "jgt r9, 30, 30001f",

        // Smaller than 30 and greater than 28
        "30000:",
        "jeq r9, 29, 283f",
        "ja 293f",

        // Larger than 30 and smaller than 32
        "30001:",
        "jeq r9, 31, 303f",
        "ja 313f",

        // Greater than 32 and smaller than 64
        "32001:",
        "jeq r9, 48, 473f",
        "jgt r9, 48, 48001f",

        // Smaller than 48 and greater than 32
        "48000:",
        "jeq r9, 40, 393f",
        "jgt r9, 40, 40001f",

        // Smaller than 40 and greater than 32
        "40000:",
        "jeq r9, 36, 353f",
        "jgt r9, 36, 36001f",

        // Smaller than 36 and greater than 32
        "36000:",
        "jeq r9, 34, 333f",
        "jgt r9, 34, 34001f",

        // Smaller than 34 and greater than 32
        "34000:",
        "jeq r9, 33, 323f",
        "ja 333f",

        // Larger than 34 and smaller than 36
        "34001:",
        "jeq r9, 35, 343f",
        "ja 353f",

        // Greater than 36 and smaller than 40
        "36001:",
        "jeq r9, 38, 373f",
        "jgt r9, 38, 38001f",

        // Smaller than 38 and greater than 36
        "38000:",
        "jeq r9, 37, 363f",
        "ja 373f",

        // Larger than 38 and smaller than 40
        "38001:",
        "jeq r9, 39, 383f",
        "ja 393f",

        // Greater than 40 and smaller than 48
        "40001:",
        "jeq r9, 44, 433f",
        "jgt r9, 44, 44001f",

        // Smaller than 44 and greater than 40
        "44000:",
        "jeq r9, 42, 413f",
        "jgt r9, 42, 42001f",

        // Smaller than 42 and greater than 40
        "42000:",
        "jeq r9, 41, 403f",
        "ja 413f",

        // Larger than 42 and smaller than 44
        "42001:",
        "jeq r9, 43, 423f",
        "ja 433f",

        // Greater than 44 and smaller than 48
        "44001:",
        "jeq r9, 46, 453f",
        "jgt r9, 46, 46001f",

        // Smaller than 46 and greater than 44
        "46000:",
        "jeq r9, 45, 443f",
        "ja 453f",

        // Larger than 46 and smaller than 48
        "46001:",
        "jeq r9, 47, 463f",
        "ja 473f",

        // Greater than 48 and smaller than 64
        "48001:",
        "jeq r9, 56, 553f",
        "jgt r9, 56, 56001f",

        // Smaller than 56 and greater than 48
        "56000:",
        "jeq r9, 52, 513f",
        "jgt r9, 52, 52001f",

        // Smaller than 52 and greater than 48
        "52000:",
        "jeq r9, 50, 493f",
        "jgt r9, 50, 50001f",

        // Smaller than 50 and greater than 48
        "50000:",
        "jeq r9, 49, 483f",
        "ja 493f",

        // Larger than 50 and smaller than 52
        "50001:",
        "jeq r9, 51, 503f",
        "ja 513f",

        // Greater than 52 and smaller than 56
        "52001:",
        "jeq r9, 54, 533f",
        "jgt r9, 54, 54001f",

        // Smaller than 54 and greater than 52
        "54000:",
        "jeq r9, 53, 523f",
        "ja 533f",

        // Larger than 54 and smaller than 56
        "54001:",
        "jeq r9, 55, 543f",
        "ja 553f",

        // Greater than 56 and smaller than 64
        "56001:",
        "jeq r9, 60, 593f",
        "jgt r9, 60, 60001f",

        // Smaller than 60 and greater than 56
        "60000:",
        "jeq r9, 58, 573f",
        "jgt r9, 58, 58001f",

        // Smaller than 58 and greater than 56
        "58000:",
        "jeq r9, 57, 563f",
        "ja 573f",

        // Larger than 58 and smaller than 60
        "58001:",
        "jeq r9, 59, 583f",
        "ja 593f",

        // Greater than 60 and smaller than 64
        "60001:",
        "jeq r9, 62, 613f",
        "jgt r9, 62, 62001f",

        // Smaller than 62 and greater than 60
        "62000:",
        "jeq r9, 61, 603f",
        "ja 613f",

        // Larger than 62 and smaller than 64
        "62001:",
        "jeq r9, 63, 623f",
        // Has to be 64 otherwise
        "ja 633f",

        /* END BINARY SEARCH "JUMP TABLE" */

        /* START INLINED CODE FOR 1-63 REMAINING ACCOUNTS */
        "633:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 631f",

        "630:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 621f",
        "ja 620f",

        "631:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 621f",
        "ja 620f",

        "623:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 621f",

        "620:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 611f",
        "ja 610f",

        "621:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 611f",
        "ja 610f",


        "613:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 611f",

        "610:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 601f",
        "ja 600f",

        "611:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 601f",
        "ja 600f",


        "603:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 601f",

        "600:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 591f",
        "ja 590f",

        "601:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 591f",
        "ja 590f",


        "593:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 591f",

        "590:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 581f",
        "ja 580f",

        "591:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 581f",
        "ja 580f",


        "583:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 581f",

        "580:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 571f",
        "ja 570f",

        "581:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 571f",
        "ja 570f",


        "573:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 571f",

        "570:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 561f",
        "ja 560f",

        "571:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 561f",
        "ja 560f",


        "563:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 561f",

        "560:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 551f",
        "ja 550f",

        "561:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 551f",
        "ja 550f",


        "553:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 551f",

        "550:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 541f",
        "ja 540f",

        "551:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 541f",
        "ja 540f",


        "543:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 541f",

        "540:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 531f",
        "ja 530f",

        "541:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 531f",
        "ja 530f",


        "533:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 531f",

        "530:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 521f",
        "ja 520f",

        "531:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 521f",
        "ja 520f",


        "523:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 521f",

        "520:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 511f",
        "ja 510f",

        "521:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 511f",
        "ja 510f",

        "513:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 511f",

        "510:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 501f",
        "ja 500f",

        "511:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 501f",
        "ja 500f",


        "503:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 501f",

        "500:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 491f",
        "ja 490f",

        "501:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 491f",
        "ja 490f",


        "493:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 491f",

        "490:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 481f",
        "ja 480f",

        "491:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 481f",
        "ja 480f",


        "483:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 481f",

        "480:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 471f",
        "ja 470f",

        "481:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 471f",
        "ja 470f",


        "473:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 471f",

        "470:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 461f",
        "ja 460f",

        "471:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 461f",
        "ja 460f",


        "463:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 461f",

        "460:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 451f",
        "ja 450f",

        "461:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 451f",
        "ja 450f",


        "453:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 451f",

        "450:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 441f",
        "ja 440f",

        "451:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 441f",
        "ja 440f",


        "443:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 441f",

        "440:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 431f",
        "ja 430f",

        "441:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 431f",
        "ja 430f",


        "433:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 431f",

        "430:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 421f",
        "ja 420f",

        "431:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 421f",
        "ja 420f",


        "423:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 421f",

        "420:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 411f",
        "ja 410f",

        "421:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 411f",
        "ja 410f",


        "413:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 411f",

        "410:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 401f",
        "ja 400f",

        "411:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 401f",
        "ja 400f",


        "403:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 401f",

        "400:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 391f",
        "ja 390f",

        "401:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 391f",
        "ja 390f",


        "393:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 391f",

        "390:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 381f",
        "ja 380f",

        "391:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 381f",
        "ja 380f",


        "383:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 381f",

        "380:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 371f",
        "ja 370f",

        "381:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 371f",
        "ja 370f",


        "373:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 371f",

        "370:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 361f",
        "ja 360f",

        "371:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 361f",
        "ja 360f",


        "363:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 361f",

        "360:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 351f",
        "ja 350f",

        "361:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 351f",
        "ja 350f",


        "353:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 351f",

        "350:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 341f",
        "ja 340f",

        "351:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 341f",
        "ja 340f",


        "343:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 341f",

        "340:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 331f",
        "ja 330f",

        "341:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 331f",
        "ja 330f",


        "333:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 331f",

        "330:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 321f",
        "ja 320f",

        "331:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 321f",
        "ja 320f",


        "323:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 321f",

        "320:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 311f",
        "ja 310f",

        "321:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 311f",
        "ja 310f",


        "313:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 311f",

        "310:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 301f",
        "ja 300f",

        "311:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 301f",
        "ja 300f",


        "303:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 301f",

        "300:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 291f",
        "ja 290f",

        "301:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 291f",
        "ja 290f",


        "293:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 291f",

        "290:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 281f",
        "ja 280f",

        "291:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 281f",
        "ja 280f",


        "283:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 281f",

        "280:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 271f",
        "ja 270f",

        "281:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 271f",
        "ja 270f",


        "273:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 271f",

        "270:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 261f",
        "ja 260f",

        "271:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 261f",
        "ja 260f",


        "263:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 261f",

        "260:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 251f",
        "ja 250f",

        "261:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 251f",
        "ja 250f",


        "253:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 251f",

        "250:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 241f",
        "ja 240f",

        "251:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 241f",
        "ja 240f",


        "243:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 241f",

        "240:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 231f",
        "ja 230f",

        "241:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 231f",
        "ja 230f",


        "233:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 231f",

        "230:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 221f",
        "ja 220f",

        "231:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 221f",
        "ja 220f",


        "223:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 221f",

        "220:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 211f",
        "ja 210f",

        "221:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 211f",
        "ja 210f",


        "213:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 211f",

        "210:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 201f",
        "ja 200f",

        "211:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 201f",
        "ja 200f",


        "203:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 201f",

        "200:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 191f",
        "ja 190f",

        "201:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 191f",
        "ja 190f",


        "193:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 191f",

        "190:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 181f",
        "ja 180f",

        "191:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 181f",
        "ja 180f",


        "183:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 181f",

        "180:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 171f",
        "ja 170f",

        "181:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 171f",
        "ja 170f",


        "173:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 171f",

        "170:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 161f",
        "ja 160f",

        "171:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 161f",
        "ja 160f",


        "163:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 161f",

        "160:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 151f",
        "ja 150f",

        "161:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 151f",
        "ja 150f",


        "153:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 151f",

        "150:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 141f",
        "ja 140f",

        "151:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 141f",
        "ja 140f",


        "143:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 141f",

        "140:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 131f",
        "ja 130f",

        "141:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 131f",
        "ja 130f",


        "133:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 131f",

        "130:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 121f",
        "ja 120f",

        "131:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 121f",
        "ja 120f",


        "123:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 121f",

        "120:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 111f",
        "ja 110f",

        "121:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 111f",
        "ja 110f",


        "113:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 111f",

        "110:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 101f",
        "ja 100f",

        "111:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 101f",
        "ja 100f",


        "103:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 101f",

        "100:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 91f",
        "ja 90f",

        "101:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 91f",
        "ja 90f",


        "93:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 91f",

        "90:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 81f",
        "ja 80f",

        "91:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 81f",
        "ja 80f",


        "83:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 81f",

        "80:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 71f",
        "ja 70f",

        "81:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 71f",
        "ja 70f",


        "73:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 71f",

        "70:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 61f",
        "ja 60f",

        "71:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 61f",
        "ja 60f",


        "63:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 61f",

        "60:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 51f",
        "ja 50f",

        "61:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 51f",
        "ja 50f",


        "53:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 51f",

        "50:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 41f",
        "ja 40f",

        "51:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 41f",
        "ja 40f",


        "43:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 41f",

        "40:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 31f",
        "ja 30f",

        "41:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 31f",
        "ja 30f",

        "33:",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 31f",

        "30:",
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 21f",
        "ja 20f",

        "31:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 21f",
        "ja 20f",

        "23:",
        // Otherwise, increment account cursor, load dup marker, jump to dup if dup
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 21f",

        // Exactly two accounts left, next one is non-dupe
        "20:",
        // Store account ptr and load account data len
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        // Advance input cursor by static data, account data, and round up to next 8
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        // Increment account cursor, load dup marker, jump to dup with one account left if dup
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 11f",
        "ja 10f",

        // Exactly two accounts left, next one is dupe
        "21:",
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        // Store in r7 and advance input cursor
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",
        // Increment account cursor, load dup marker, jump to dup with one account left if dup
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 11f",
        "ja 10f",

        "13:",
        // Otherwise, increment account cursor, load dup marker, jump to dup if dup
        "add64 r7, 8",
        "ldxb r6, [r1 + 8]",
        "jne r6, 255, 11f",

        // Exactly one account left, non-dupe
        "10:", 
        // Store account ptr and load account data len
        "stxdw [r7 + 0], r1",
        "ldxdw r8, [r1 + 72 + 8]",
        // Advance input cursor by static data, account data, and round up to next 8
        "add64 r1, {account_total}",
        "add64 r1, r8",
        "and64 r1, 0xFFFFFFFFFFFFFFF8",
        "ja 1006f",

        // Exactly one account left, dupe
        "11:", 
        // Calculate index, load account into r6
        "mul64 r6, 8",
        "lddw r8, {accounts_ptr}",
        "add64 r6, r8",
        "ldxdw r6, [r6 + 0]",
        // Store in r7 and advance input cursor
        "stxdw [r7 + 0], r6",
        "add64 r1, 8",

        /* END INLINED CODE FOR 1-63 REMAINING ACCOUNTS */

        // Finished
        "1006:",
        "add64 r1, 8",

        inout("r1") input,
        out("r9") num_accounts,
        accounts_ptr = const ACCOUNTS_PTR,
        account_total = const TOTAL_ACCOUNT_DATA_TO_SKIP,
        options(nostack),
    );

    let instruction_data_len = *(input as *const u64) as usize;
    input = input.add(core::mem::size_of::<u64>());

    let data = core::slice::from_raw_parts(input, instruction_data_len);
    input = input.add(instruction_data_len);

    let program_id: &Pubkey = &*(input as *const Pubkey);

    let accounts = core::slice::from_raw_parts(
        ACCOUNTS_PTR as *const AccountInfo,
        num_accounts.assume_init(),
    );

    // sol_log_64(data.len() as u64, accounts.len() as u64, 0, 0, 0);

    process(program_id, accounts, data)
}

#[inline(always)]
#[allow(unused)]
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> u32 {
    0
}
