open Alcotest
open Lib.Day01a

let () =
  run "advent of code"
    [
      ( "day 01 a",
        [
          test_case "sample" `Quick (fun () ->
              Alcotest.check int "expected value" (do_it "day01a-sample.txt") 11);
          test_case "actual" `Quick (fun () ->
              Alcotest.check int "expected value" (do_it "day01a.txt") 1319616);
        ] );
    ]
