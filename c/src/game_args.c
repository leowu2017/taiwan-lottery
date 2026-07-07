#include <string.h>

#include <taiwan_lottery/query.h>

#include "game_args.h"

int taiwan_lottery_parse_game_arg(const char *value) {
    if (strcmp(value, "super-lotto638") == 0 || strcmp(value, "superlotto638") == 0 || strcmp(value, "5134") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_SUPER_LOTTO_638;
    }
    if (strcmp(value, "lotto649") == 0 || strcmp(value, "5118") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_LOTTO_649;
    }
    if (strcmp(value, "daily539") == 0 || strcmp(value, "5120") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_DAILY_539;
    }
    if (strcmp(value, "3d") == 0 || strcmp(value, "2108") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_3D;
    }
    if (strcmp(value, "4d") == 0 || strcmp(value, "2109") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_4D;
    }
    if (strcmp(value, "49m6") == 0 || strcmp(value, "1121") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_49M6;
    }
    if (strcmp(value, "39m5") == 0 || strcmp(value, "1197") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_39M5;
    }
    if (strcmp(value, "38m6") == 0 || strcmp(value, "5122") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_38M6;
    }
    if (strcmp(value, "1224") == 0 || strcmp(value, "5290") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_1224;
    }
    if (strcmp(value, "740") == 0 || strcmp(value, "2300") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_740;
    }
    if (strcmp(value, "tic-tac-toe") == 0 || strcmp(value, "tictactoe") == 0 || strcmp(value, "2400") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_TIC_TAC_TOE;
    }
    if (strcmp(value, "638") == 0 || strcmp(value, "2500") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_638;
    }
    if (strcmp(value, "bingo-bingo") == 0 || strcmp(value, "bingobingo") == 0 || strcmp(value, "1102") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_BINGO_BINGO;
    }

    return -1;
}