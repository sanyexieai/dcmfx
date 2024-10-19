import dcmfx_character_set/internal/gb_18030
import gleam/list
import gleam/string
import gleeunit/should

pub fn decode_next_codepoint_test() {
  [
    #(<<0xD7, 0xD4>>, 0x81EA),
    #(<<0xDB, 0xFA>>, 0x57F4),
    #(<<0x82, 0x7F>>, 0xFFFD),
    #(<<0xA8, 0xBC>>, 0xE7C7),
    #(<<0x81, 0x30, 0x84, 0x36>>, 0x00A5),
    #(<<0x81, 0x30, 0x84, 0x38>>, 0x00A9),
    #(<<0x81, 0x30, 0x85, 0x35>>, 0x00B2),
    #(<<0x81, 0x30, 0x86, 0x30>>, 0x00B8),
    #(<<0x81, 0x30, 0x89, 0x31>>, 0x00D8),
    #(<<0x81, 0x30, 0x89, 0x39>>, 0x00E2),
    #(<<0x81, 0x30, 0x8A, 0x35>>, 0x00EB),
    #(<<0x81, 0x30, 0x8A, 0x36>>, 0x00EE),
    #(<<0x81, 0x30, 0x8B, 0x30>>, 0x00F4),
    #(<<0x81, 0x30, 0x8B, 0x33>>, 0x00F8),
    #(<<0x81, 0x30, 0x8B, 0x34>>, 0x00FB),
    #(<<0x81, 0x30, 0x8B, 0x35>>, 0x00FD),
    #(<<0x81, 0x30, 0x8B, 0x39>>, 0x0102),
    #(<<0x81, 0x30, 0x8D, 0x36>>, 0x0114),
    #(<<0x81, 0x30, 0x8E, 0x33>>, 0x011C),
    #(<<0x81, 0x30, 0x8F, 0x38>>, 0x012C),
    #(<<0x81, 0x30, 0x92, 0x32>>, 0x0145),
    #(<<0x81, 0x30, 0x92, 0x35>>, 0x0149),
    #(<<0x81, 0x30, 0x92, 0x39>>, 0x014E),
    #(<<0x81, 0x30, 0x95, 0x38>>, 0x016C),
    #(<<0x81, 0x30, 0x9F, 0x36>>, 0x01CF),
    #(<<0x81, 0x30, 0x9F, 0x37>>, 0x01D1),
    #(<<0x81, 0x30, 0x9F, 0x38>>, 0x01D3),
    #(<<0x81, 0x30, 0x9F, 0x39>>, 0x01D5),
    #(<<0x81, 0x30, 0xA0, 0x30>>, 0x01D7),
    #(<<0x81, 0x30, 0xA0, 0x31>>, 0x01D9),
    #(<<0x81, 0x30, 0xA0, 0x32>>, 0x01DB),
    #(<<0x81, 0x30, 0xA0, 0x33>>, 0x01DD),
    #(<<0x81, 0x30, 0xA3, 0x31>>, 0x01FA),
    #(<<0x81, 0x30, 0xAB, 0x38>>, 0x0252),
    #(<<0x81, 0x30, 0xAD, 0x33>>, 0x0262),
    #(<<0x81, 0x30, 0xB7, 0x34>>, 0x02C8),
    #(<<0x81, 0x30, 0xB7, 0x35>>, 0x02CC),
    #(<<0x81, 0x30, 0xB8, 0x38>>, 0x02DA),
    #(<<0x81, 0x30, 0xCB, 0x31>>, 0x03A2),
    #(<<0x81, 0x30, 0xCB, 0x32>>, 0x03AA),
    #(<<0x81, 0x30, 0xCB, 0x39>>, 0x03C2),
    #(<<0x81, 0x30, 0xCC, 0x30>>, 0x03CA),
    #(<<0x81, 0x30, 0xD1, 0x35>>, 0x0402),
    #(<<0x81, 0x30, 0xD2, 0x39>>, 0x0450),
    #(<<0x81, 0x30, 0xD3, 0x30>>, 0x0452),
    #(<<0x81, 0x36, 0xA5, 0x32>>, 0x2011),
    #(<<0x81, 0x36, 0xA5, 0x34>>, 0x2017),
    #(<<0x81, 0x36, 0xA5, 0x35>>, 0x201A),
    #(<<0x81, 0x36, 0xA5, 0x37>>, 0x201E),
    #(<<0x81, 0x36, 0xA6, 0x34>>, 0x2027),
    #(<<0x81, 0x36, 0xA7, 0x33>>, 0x2031),
    #(<<0x81, 0x36, 0xA7, 0x34>>, 0x2034),
    #(<<0x81, 0x36, 0xA7, 0x35>>, 0x2036),
    #(<<0x81, 0x36, 0xA8, 0x30>>, 0x203C),
    #(<<0x81, 0x36, 0xB3, 0x32>>, 0x20AD),
    #(<<0x81, 0x36, 0xBB, 0x38>>, 0x2104),
    #(<<0x81, 0x36, 0xBB, 0x39>>, 0x2106),
    #(<<0x81, 0x36, 0xBC, 0x32>>, 0x210A),
    #(<<0x81, 0x36, 0xBD, 0x34>>, 0x2117),
    #(<<0x81, 0x36, 0xBE, 0x34>>, 0x2122),
    #(<<0x81, 0x36, 0xC4, 0x36>>, 0x216C),
    #(<<0x81, 0x36, 0xC5, 0x30>>, 0x217A),
    #(<<0x81, 0x36, 0xC7, 0x32>>, 0x2194),
    #(<<0x81, 0x36, 0xC7, 0x34>>, 0x219A),
    #(<<0x81, 0x36, 0xD2, 0x34>>, 0x2209),
    #(<<0x81, 0x36, 0xD3, 0x30>>, 0x2210),
    #(<<0x81, 0x36, 0xD3, 0x31>>, 0x2212),
    #(<<0x81, 0x36, 0xD3, 0x34>>, 0x2216),
    #(<<0x81, 0x36, 0xD3, 0x38>>, 0x221B),
    #(<<0x81, 0x36, 0xD4, 0x30>>, 0x2221),
    #(<<0x81, 0x36, 0xD4, 0x32>>, 0x2224),
    #(<<0x81, 0x36, 0xD4, 0x33>>, 0x2226),
    #(<<0x81, 0x36, 0xD4, 0x34>>, 0x222C),
    #(<<0x81, 0x36, 0xD4, 0x36>>, 0x222F),
    #(<<0x81, 0x36, 0xD5, 0x31>>, 0x2238),
    #(<<0x81, 0x36, 0xD5, 0x36>>, 0x223E),
    #(<<0x81, 0x36, 0xD6, 0x36>>, 0x2249),
    #(<<0x81, 0x36, 0xD6, 0x39>>, 0x224D),
    #(<<0x81, 0x36, 0xD7, 0x34>>, 0x2253),
    #(<<0x81, 0x36, 0xD8, 0x37>>, 0x2262),
    #(<<0x81, 0x36, 0xD8, 0x39>>, 0x2268),
    #(<<0x81, 0x36, 0xD9, 0x35>>, 0x2270),
    #(<<0x81, 0x36, 0xDD, 0x32>>, 0x2296),
    #(<<0x81, 0x36, 0xDD, 0x35>>, 0x229A),
    #(<<0x81, 0x36, 0xDE, 0x36>>, 0x22A6),
    #(<<0x81, 0x36, 0xE1, 0x31>>, 0x22C0),
    #(<<0x81, 0x36, 0xE9, 0x33>>, 0x2313),
    #(<<0x81, 0x37, 0x8C, 0x36>>, 0x246A),
    #(<<0x81, 0x37, 0x8D, 0x36>>, 0x249C),
    #(<<0x81, 0x37, 0x97, 0x36>>, 0x254C),
    #(<<0x81, 0x37, 0x98, 0x30>>, 0x2574),
    #(<<0x81, 0x37, 0x99, 0x33>>, 0x2590),
    #(<<0x81, 0x37, 0x99, 0x36>>, 0x2596),
    #(<<0x81, 0x37, 0x9A, 0x36>>, 0x25A2),
    #(<<0x81, 0x37, 0x9C, 0x32>>, 0x25B4),
    #(<<0x81, 0x37, 0x9D, 0x30>>, 0x25BE),
    #(<<0x81, 0x37, 0x9D, 0x38>>, 0x25C8),
    #(<<0x81, 0x37, 0x9E, 0x31>>, 0x25CC),
    #(<<0x81, 0x37, 0x9E, 0x33>>, 0x25D0),
    #(<<0x81, 0x37, 0xA0, 0x31>>, 0x25E6),
    #(<<0x81, 0x37, 0xA3, 0x32>>, 0x2607),
    #(<<0x81, 0x37, 0xA3, 0x34>>, 0x260A),
    #(<<0x81, 0x37, 0xA8, 0x38>>, 0x2641),
    #(<<0x81, 0x37, 0xA8, 0x39>>, 0x2643),
    #(<<0x81, 0x38, 0xFD, 0x39>>, 0x2E82),
    #(<<0x81, 0x38, 0xFE, 0x31>>, 0x2E85),
    #(<<0x81, 0x38, 0xFE, 0x34>>, 0x2E89),
    #(<<0x81, 0x38, 0xFE, 0x36>>, 0x2E8D),
    #(<<0x81, 0x39, 0x81, 0x36>>, 0x2E98),
    #(<<0x81, 0x39, 0x83, 0x31>>, 0x2EA8),
    #(<<0x81, 0x39, 0x83, 0x33>>, 0x2EAB),
    #(<<0x81, 0x39, 0x83, 0x36>>, 0x2EAF),
    #(<<0x81, 0x39, 0x84, 0x30>>, 0x2EB4),
    #(<<0x81, 0x39, 0x84, 0x32>>, 0x2EB8),
    #(<<0x81, 0x39, 0x84, 0x35>>, 0x2EBC),
    #(<<0x81, 0x39, 0x85, 0x39>>, 0x2ECB),
    #(<<0x81, 0x39, 0xA3, 0x32>>, 0x2FFC),
    #(<<0x81, 0x39, 0xA3, 0x36>>, 0x3004),
    #(<<0x81, 0x39, 0xA3, 0x37>>, 0x3018),
    #(<<0x81, 0x39, 0xA4, 0x32>>, 0x301F),
    #(<<0x81, 0x39, 0xA4, 0x34>>, 0x302A),
    #(<<0x81, 0x39, 0xA6, 0x34>>, 0x303F),
    #(<<0x81, 0x39, 0xA6, 0x36>>, 0x3094),
    #(<<0x81, 0x39, 0xA7, 0x33>>, 0x309F),
    #(<<0x81, 0x39, 0xA7, 0x35>>, 0x30F7),
    #(<<0x81, 0x39, 0xA8, 0x30>>, 0x30FF),
    #(<<0x81, 0x39, 0xA8, 0x36>>, 0x312A),
    #(<<0x81, 0x39, 0xC1, 0x32>>, 0x322A),
    #(<<0x81, 0x39, 0xC1, 0x39>>, 0x3232),
    #(<<0x81, 0x39, 0xCD, 0x32>>, 0x32A4),
    #(<<0x81, 0x39, 0xE4, 0x36>>, 0x3390),
    #(<<0x81, 0x39, 0xE5, 0x38>>, 0x339F),
    #(<<0x81, 0x39, 0xE6, 0x30>>, 0x33A2),
    #(<<0x81, 0x39, 0xE9, 0x34>>, 0x33C5),
    #(<<0x81, 0x39, 0xEA, 0x33>>, 0x33CF),
    #(<<0x81, 0x39, 0xEA, 0x35>>, 0x33D3),
    #(<<0x81, 0x39, 0xEA, 0x37>>, 0x33D6),
    #(<<0x81, 0x39, 0xF6, 0x30>>, 0x3448),
    #(<<0x81, 0x39, 0xFA, 0x33>>, 0x3474),
    #(<<0x82, 0x30, 0x9A, 0x31>>, 0x359F),
    #(<<0x82, 0x30, 0xA5, 0x32>>, 0x360F),
    #(<<0x82, 0x30, 0xA6, 0x33>>, 0x361B),
    #(<<0x82, 0x30, 0xF2, 0x38>>, 0x3919),
    #(<<0x82, 0x30, 0xFB, 0x33>>, 0x396F),
    #(<<0x82, 0x31, 0x86, 0x39>>, 0x39D1),
    #(<<0x82, 0x31, 0x88, 0x33>>, 0x39E0),
    #(<<0x82, 0x31, 0x97, 0x30>>, 0x3A74),
    #(<<0x82, 0x31, 0xAC, 0x38>>, 0x3B4F),
    #(<<0x82, 0x31, 0xC9, 0x35>>, 0x3C6F),
    #(<<0x82, 0x31, 0xD4, 0x38>>, 0x3CE1),
    #(<<0x82, 0x32, 0xAF, 0x33>>, 0x4057),
    #(<<0x82, 0x32, 0xC9, 0x37>>, 0x4160),
    #(<<0x82, 0x32, 0xF8, 0x38>>, 0x4338),
    #(<<0x82, 0x33, 0x86, 0x34>>, 0x43AD),
    #(<<0x82, 0x33, 0x86, 0x38>>, 0x43B2),
    #(<<0x82, 0x33, 0x8B, 0x31>>, 0x43DE),
    #(<<0x82, 0x33, 0xA3, 0x39>>, 0x44D7),
    #(<<0x82, 0x33, 0xC9, 0x32>>, 0x464D),
    #(<<0x82, 0x33, 0xCB, 0x32>>, 0x4662),
    #(<<0x82, 0x33, 0xDE, 0x35>>, 0x4724),
    #(<<0x82, 0x33, 0xDF, 0x30>>, 0x472A),
    #(<<0x82, 0x33, 0xE7, 0x32>>, 0x477D),
    #(<<0x82, 0x33, 0xE8, 0x38>>, 0x478E),
    #(<<0x82, 0x34, 0x96, 0x39>>, 0x4948),
    #(<<0x82, 0x34, 0x9B, 0x39>>, 0x497B),
    #(<<0x82, 0x34, 0x9C, 0x31>>, 0x497E),
    #(<<0x82, 0x34, 0x9C, 0x35>>, 0x4984),
    #(<<0x82, 0x34, 0x9C, 0x36>>, 0x4987),
    #(<<0x82, 0x34, 0x9E, 0x36>>, 0x499C),
    #(<<0x82, 0x34, 0x9E, 0x39>>, 0x49A0),
    #(<<0x82, 0x34, 0xA1, 0x31>>, 0x49B8),
    #(<<0x82, 0x34, 0xE7, 0x34>>, 0x4C78),
    #(<<0x82, 0x34, 0xEB, 0x33>>, 0x4CA4),
    #(<<0x82, 0x34, 0xF6, 0x34>>, 0x4D1A),
    #(<<0x82, 0x35, 0x87, 0x32>>, 0x4DAF),
    #(<<0x82, 0x35, 0x8F, 0x33>>, 0x9FA6),
    #(<<0x83, 0x36, 0xC7, 0x39>>, 0xE76C),
    #(<<0x83, 0x36, 0xC8, 0x30>>, 0xE7C8),
    #(<<0x83, 0x36, 0xC8, 0x31>>, 0xE7E7),
    #(<<0x83, 0x36, 0xC9, 0x34>>, 0xE815),
    #(<<0x83, 0x36, 0xC9, 0x35>>, 0xE819),
    #(<<0x83, 0x36, 0xCA, 0x30>>, 0xE81F),
    #(<<0x83, 0x36, 0xCA, 0x37>>, 0xE827),
    #(<<0x83, 0x36, 0xCB, 0x31>>, 0xE82D),
    #(<<0x83, 0x36, 0xCB, 0x35>>, 0xE833),
    #(<<0x83, 0x36, 0xCC, 0x33>>, 0xE83C),
    #(<<0x83, 0x36, 0xCD, 0x30>>, 0xE844),
    #(<<0x83, 0x36, 0xCE, 0x36>>, 0xE856),
    #(<<0x83, 0x36, 0xD0, 0x30>>, 0xE865),
    #(<<0x84, 0x30, 0x85, 0x35>>, 0xF92D),
    #(<<0x84, 0x30, 0x8D, 0x31>>, 0xF97A),
    #(<<0x84, 0x30, 0x8F, 0x38>>, 0xF996),
    #(<<0x84, 0x30, 0x97, 0x39>>, 0xF9E8),
    #(<<0x84, 0x30, 0x98, 0x38>>, 0xF9F2),
    #(<<0x84, 0x30, 0x9B, 0x34>>, 0xFA10),
    #(<<0x84, 0x30, 0x9B, 0x35>>, 0xFA12),
    #(<<0x84, 0x30, 0x9B, 0x36>>, 0xFA15),
    #(<<0x84, 0x30, 0x9B, 0x39>>, 0xFA19),
    #(<<0x84, 0x30, 0x9C, 0x35>>, 0xFA22),
    #(<<0x84, 0x30, 0x9C, 0x36>>, 0xFA25),
    #(<<0x84, 0x30, 0x9C, 0x38>>, 0xFA2A),
    #(<<0x84, 0x31, 0x85, 0x38>>, 0xFE32),
    #(<<0x84, 0x31, 0x85, 0x39>>, 0xFE45),
    #(<<0x84, 0x31, 0x86, 0x33>>, 0xFE53),
    #(<<0x84, 0x31, 0x86, 0x34>>, 0xFE58),
    #(<<0x84, 0x31, 0x86, 0x35>>, 0xFE67),
    #(<<0x84, 0x31, 0x86, 0x36>>, 0xFE6C),
    #(<<0x84, 0x31, 0x95, 0x35>>, 0xFF5F),
    #(<<0x84, 0x31, 0xA2, 0x34>>, 0xFFE6),
    #(<<0x84, 0x31, 0xA4, 0x38>>, 0xFFFD),
    #(<<0x90, 0x30, 0x81, 0x30>>, 0x10000),
    #(<<0xE3, 0x32, 0x9A, 0x35>>, 0x10FFFF),
    #(<<0xE3, 0x32, 0x9A, 0x36>>, 0xFFFD),
    #(<<0x86, 0x32, 0xAA, 0x35>>, 0xFFFD),
    #(<<0xFF, 0xFF, 0xFF, 0xFF>>, 0xFFFD),
  ]
  |> list.each(fn(x) {
    let #(bytes, expected_codepoint) = x

    let assert Ok(r) =
      bytes
      |> gb_18030.decode_next_codepoint

    let assert Ok(expected_codepoint) = string.utf_codepoint(expected_codepoint)

    should.equal(r.0, expected_codepoint)
  })

  <<>>
  |> gb_18030.decode_next_codepoint
  |> should.equal(Error(Nil))
}
