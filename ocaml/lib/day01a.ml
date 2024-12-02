let join_paths paths =
  List.fold_left (fun result next -> Filename.concat result next) "" paths

let realpath path =
  let proc_out = Unix.open_process_in ("realpath " ^ Filename.quote path) in
  try
    let line = input_line proc_out in
    close_in proc_out;
    line
  with e ->
    close_in_noerr proc_out;
    raise e

let safe_readline input =
  try
    let result = input_line input in
    Some result
  with End_of_file -> None

let rec safe_read_all_lines input results =
  let line = safe_readline input in
  match line with
  | Some line ->
      let results = line :: results in
      safe_read_all_lines input results
  | None -> results

let read_lines path =
  let file = open_in path in
  try
    let results = safe_read_all_lines file [] in
    close_in file;
    List.rev results
  with e ->
    close_in_noerr file;
    raise e

let read_puzzle_lines path =
  let path =
    join_paths
      [
        Filename.current_dir_name; ".."; ".."; ".."; ".."; "puzzle-inputs"; path;
      ]
  in
  let path = realpath path in
  read_lines path

let match_all_lines_return_capture_groups r lines =
  List.filter_map
    (fun line ->
      match Pcre2.split ~rex:r line with
      | _ :: result -> Some result
      | _ -> None)
    lines

let rec unzip_lists lines left_results right_results =
  match lines with
  | [ left; right ] :: remainder ->
      let left = left |> int_of_string in
      let right = right |> int_of_string in
      let left_results, right_results =
        unzip_lists remainder left_results right_results
      in
      (left :: left_results, right :: right_results)
  | _ :: _ ->
      print_endline "bad line found, should be impossible";
      (left_results, right_results)
  | _ -> (left_results, right_results)

let do_it path =
  let r = Pcre2.regexp {|^(\d+)\s+(\d+)$|} in
  let lines = read_puzzle_lines path in
  let lines = match_all_lines_return_capture_groups r lines in
  let left, right = unzip_lists lines [] [] in

  let left = List.sort (fun a b -> a - b) left in
  let right = List.sort (fun a b -> a - b) right in

  let differences =
    List.combine left right |> List.map (fun (a, b) -> abs (a - b))
  in
  List.fold_left (fun result x -> result + x) 0 differences
