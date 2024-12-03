open Alcotest
open Lib.Day01a

let () =
  run "advent of code"
    [
      ( "day 01 a",
        [
          test_case "sample" `Quick (fun () ->
              Alcotest.check int "expected value" (do_it "day01-sample.txt") 11);
          test_case "actual" `Quick (fun () ->
              Alcotest.check int "expected value" (do_it "day01.txt") 1319616);
        ] );
    ]
