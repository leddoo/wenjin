;; Test `loop` opcode

(module
  (func (export "params-shifter") (result i32)
    ;; counter.
    (local $x i32)
    (local.set $x (i32.const 0))
    (i32.const 1)
    (i32.const 2)
    (loop (param i32 i32) (result i32 i32)
      ;; push x
      (local.get $x)
      ;; increment counter.
      (local.set $x (i32.add (local.get $x) (i32.const 1)))
      ;; loop if counter < 10.
      ;; and also, on jump, turn [1, 2, x] into [2, x]
      (br_if 0 (i32.lt_u (local.get $x) (i32.const 10)))
      ;; drop x.
      (drop)
    )
    (i32.add)
  )
)

(assert_return (invoke "params-shifter") (i32.const 15)) ;; 7 + 8

