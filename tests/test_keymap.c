/*
 * test_keymap.c — unit tests for the ASCII -> Linux keycode mapping.
 * Pure logic tests, no uinput/display required.
 */

#include "testutil.h"
#include <stdint.h>
#include <linux/input-event-codes.h>
#include "../keymap.h"

#define EXPECT(layout, ch, expected, failed) \
    ASSERT_EQ((int32_t)(expected), keymap_lookup((layout), (ch)), "keymap_lookup(" layout ", " #ch ")", failed)

/* ── US layout: must match the original hardcoded table ─────────────────── */

void test_us_layout(int *failed) {
    EXPECT("us", 'a', KEY_A, failed);
    EXPECT("us", 'A', KEY_A | FLAG_UPPERCASE, failed);
    EXPECT("us", '1', KEY_1, failed);
    EXPECT("us", '!', KEY_1 | FLAG_UPPERCASE, failed);
    EXPECT("us", '\'', KEY_APOSTROPHE, failed);
    EXPECT("us", '?', KEY_SLASH | FLAG_UPPERCASE, failed);
    EXPECT("us", '"', KEY_APOSTROPHE | FLAG_UPPERCASE, failed);
    EXPECT("us", '@', KEY_2 | FLAG_UPPERCASE, failed);
    EXPECT("us", '^', KEY_6 | FLAG_UPPERCASE, failed);
    EXPECT("us", '~', KEY_GRAVE | FLAG_UPPERCASE, failed);
    EXPECT("us", '`', KEY_GRAVE, failed);
    EXPECT("us", '\\', KEY_BACKSLASH, failed);
    EXPECT("us", '<', KEY_COMMA | FLAG_UPPERCASE, failed);
    EXPECT("us", '{', KEY_LEFTBRACE | FLAG_UPPERCASE, failed);
    EXPECT("us", '|', KEY_BACKSLASH | FLAG_UPPERCASE, failed);
    EXPECT("us", ' ', KEY_SPACE, failed);
}

/* ── Danish layout: the two reported bugs ────────────────────────────────── */

void test_dk_layout_reported_bugs(int *failed) {
    EXPECT("dk", '\'', KEY_BACKSLASH, failed);             /* was being typed as o-slash */
    EXPECT("dk", '?', KEY_MINUS | FLAG_UPPERCASE, failed); /* was being typed as underscore */
}

/* ── Danish layout: letters and digits share physical keys with US ──────── */

void test_dk_layout_letters_and_digits(int *failed) {
    EXPECT("dk", 'a', KEY_A, failed);
    EXPECT("dk", 'A', KEY_A | FLAG_UPPERCASE, failed);
    EXPECT("dk", 'z', KEY_Z, failed);
    EXPECT("dk", 'Z', KEY_Z | FLAG_UPPERCASE, failed);
    EXPECT("dk", '1', KEY_1, failed);
    EXPECT("dk", '!', KEY_1 | FLAG_UPPERCASE, failed);
}

/* ── Danish layout: shifted symbols ──────────────────────────────────────── */

void test_dk_layout_shifted_symbols(int *failed) {
    EXPECT("dk", '"', KEY_2 | FLAG_UPPERCASE, failed);
    EXPECT("dk", '#', KEY_3 | FLAG_UPPERCASE, failed);
    EXPECT("dk", '%', KEY_5 | FLAG_UPPERCASE, failed);
    EXPECT("dk", '&', KEY_6 | FLAG_UPPERCASE, failed);
    EXPECT("dk", '6', KEY_6, failed);
    EXPECT("dk", '/', KEY_7 | FLAG_UPPERCASE, failed); /* shift+7 */
    EXPECT("dk", '7', KEY_7, failed);
    EXPECT("dk", '(', KEY_8 | FLAG_UPPERCASE, failed);
    EXPECT("dk", ')', KEY_9 | FLAG_UPPERCASE, failed);
    EXPECT("dk", '=', KEY_0 | FLAG_UPPERCASE, failed);
    EXPECT("dk", '+', KEY_MINUS, failed);
    EXPECT("dk", '*', KEY_BACKSLASH | FLAG_UPPERCASE, failed);
    EXPECT("dk", ';', KEY_COMMA | FLAG_UPPERCASE, failed);
    EXPECT("dk", ':', KEY_DOT | FLAG_UPPERCASE, failed);
    EXPECT("dk", '<', KEY_102ND, failed);
    EXPECT("dk", '>', KEY_102ND | FLAG_UPPERCASE, failed);
    EXPECT("dk", '-', KEY_SLASH, failed);
    EXPECT("dk", '_', KEY_SLASH | FLAG_UPPERCASE, failed);
}

/* ── Danish layout: AltGr characters ─────────────────────────────────────── */

void test_dk_layout_altgr(int *failed) {
    EXPECT("dk", '@', KEY_2 | FLAG_ALTGR, failed);
    EXPECT("dk", '$', KEY_4 | FLAG_ALTGR, failed);
    EXPECT("dk", '{', KEY_7 | FLAG_ALTGR, failed);
    EXPECT("dk", '[', KEY_8 | FLAG_ALTGR, failed);
    EXPECT("dk", ']', KEY_9 | FLAG_ALTGR, failed);
    EXPECT("dk", '}', KEY_0 | FLAG_ALTGR, failed);
    EXPECT("dk", '\\', KEY_102ND | FLAG_ALTGR, failed);
    EXPECT("dk", '|', KEY_EQUAL | FLAG_ALTGR, failed);
}

/* ── Danish layout: dead keys (emitted as key + space) ───────────────────── */

void test_dk_layout_dead_keys(int *failed) {
    EXPECT("dk", '`', KEY_EQUAL | FLAG_UPPERCASE | FLAG_DEADKEY, failed);
    EXPECT("dk", '^', KEY_RIGHTBRACE | FLAG_UPPERCASE | FLAG_DEADKEY, failed);
    EXPECT("dk", '~', KEY_RIGHTBRACE | FLAG_ALTGR | FLAG_DEADKEY, failed);
}

/* ── Non-ASCII and unknown layouts ────────────────────────────────────────── */

void test_non_ascii_falls_back_to_clipboard(int *failed) {
    /* Non-ASCII falls back to the clipboard path in xhisper.sh. */
    EXPECT("us", 0xC3, -1, failed);
    EXPECT("dk", 0xE6, -1, failed);
}

void test_unknown_layout_falls_back_to_us(int *failed) {
    EXPECT("gibberish", '\'', KEY_APOSTROPHE, failed);
    EXPECT("gibberish", '?', KEY_SLASH | FLAG_UPPERCASE, failed);
}

/* ── main ─────────────────────────────────────────────────────────────────── */

int main(void) {
    int failed = 0;

    TEST_SUITE_BEGIN("keymap");

    test_us_layout(&failed);
    test_dk_layout_reported_bugs(&failed);
    test_dk_layout_letters_and_digits(&failed);
    test_dk_layout_shifted_symbols(&failed);
    test_dk_layout_altgr(&failed);
    test_dk_layout_dead_keys(&failed);
    test_non_ascii_falls_back_to_clipboard(&failed);
    test_unknown_layout_falls_back_to_us(&failed);

    TEST_SUITE_END(failed);
}
