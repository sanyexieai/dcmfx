import dcmfx_core/transfer_syntax
import gleam/list
import gleeunit/should

const transfer_syntax_uids = [
  "1.2.840.10008.1.2", "1.2.840.10008.1.2.1", "1.2.840.10008.1.2.1.98",
  "1.2.840.10008.1.2.1.99", "1.2.840.10008.1.2.2", "1.2.840.10008.1.2.4.50",
  "1.2.840.10008.1.2.4.51", "1.2.840.10008.1.2.4.57", "1.2.840.10008.1.2.4.70",
  "1.2.840.10008.1.2.4.80", "1.2.840.10008.1.2.4.81", "1.2.840.10008.1.2.4.90",
  "1.2.840.10008.1.2.4.91", "1.2.840.10008.1.2.4.92", "1.2.840.10008.1.2.4.93",
  "1.2.840.10008.1.2.4.94", "1.2.840.10008.1.2.4.95", "1.2.840.10008.1.2.4.100",
  "1.2.840.10008.1.2.4.100.1", "1.2.840.10008.1.2.4.101",
  "1.2.840.10008.1.2.4.101.1", "1.2.840.10008.1.2.4.102",
  "1.2.840.10008.1.2.4.102.1", "1.2.840.10008.1.2.4.103",
  "1.2.840.10008.1.2.4.103.1", "1.2.840.10008.1.2.4.104",
  "1.2.840.10008.1.2.4.104.1", "1.2.840.10008.1.2.4.105",
  "1.2.840.10008.1.2.4.105.1", "1.2.840.10008.1.2.4.106",
  "1.2.840.10008.1.2.4.106.1", "1.2.840.10008.1.2.4.107",
  "1.2.840.10008.1.2.4.108", "1.2.840.10008.1.2.4.201",
  "1.2.840.10008.1.2.4.202", "1.2.840.10008.1.2.4.203",
  "1.2.840.10008.1.2.4.204", "1.2.840.10008.1.2.4.205", "1.2.840.10008.1.2.5",
  "1.2.840.10008.1.2.7.1", "1.2.840.10008.1.2.7.2", "1.2.840.10008.1.2.7.3",
]

pub fn all_test() {
  transfer_syntax.all
  |> list.map(fn(ts) { ts.uid })
  |> should.equal(transfer_syntax_uids)
}

pub fn from_uid_test() {
  transfer_syntax_uids
  |> list.each(fn(uid) {
    uid
    |> transfer_syntax.from_uid
    |> should.be_ok
  })

  "1.2.3.4"
  |> transfer_syntax.from_uid
  |> should.be_error
}
