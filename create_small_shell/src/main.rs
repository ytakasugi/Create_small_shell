use std::env;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

fn main(){
    loop {
        // プロンプトとして`>`文字を使用
        // read_lineの前に`print`されるように、明示的にフラッシュする必要がある
        print!("> ");
        stdout().flush().unwrap();

        // 新しい空のStringを作成
        let mut input = String::new();
        // 現在のプロセスの標準入力への新しい処理を構築し、
        // 処理をロックして入力行を読み、指定されたバッファに追加する
        stdin().read_line(&mut input).unwrap();

        // `read_line`は末尾に改行を残っているので、`trim`で削除する。
        // その後、`split_whitespace`でホワイトスペースで分割する
        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next()  {

            // `read_line`は末尾に改行を残っているので、`trim`で削除する。
            // その後、`split_whitespace`でホワイトスペースで分割する
            let mut parts = command.trim().split_whitespace();
            // `next`メソッドで、ホワイトスペース以降の文字列を引数として扱う
            let command = parts.next().unwrap();
            let args = parts;

            match command {
                "cd" => {
                    // 新しいディレクトリが提供されていない場合、デフォルトで「/」が使用される。
                    let new_dir = args.peekable().peek().map_or("/", |x| *x);
                    // 文字列スライスを`Path`スライスとして直接ラップする
                    let root = Path::new(new_dir);
                    // もし、エラーの場合は、現在の作業ディレクトリを、指定したパスに返す
                    if let Err(e) = env::set_current_dir(&root) {
                        eprintln!("{}", e);
                    }
                    previous_command = None;
                },
                "exit" => return,
                command => {
                    let stdin = previous_command
                        .map_or(Stdio::inherit(),
                                |output: Child| Stdio::from(output.stdout.unwrap()));

                    let stdout = if commands.peek().is_some() {
                        // このコマンドの後ろには、もう一つのコマンドがあります。
                        // 次のコマンドに出力を送るための準備
                        Stdio::piped()
                    } else {
                        // このコマンドの後ろにはもうコマンドはありません。
                        // シェルの標準出力に出力を送る
                        Stdio::inherit()
                    };

                    // 新しい`Command`を構築し、その処理を返す
                    let output = Command::new(command)
                                .args(args)
                                .stdin(stdin)
                                .stdout(stdout)
                                .spawn();

                    match output {
                        Ok(output) => { 
                            previous_command = Some(output); 
                        },
                        Err(e) => {
                            previous_command = None;
                            eprintln!("{}", e);
                        },
                    };
                }
            }
        }
        
        if let Some(mut final_command) = previous_command {
            // このコマンドが完了するまで、他のコマンドを受け付けない。
            final_command.wait().unwrap();
        }
    }
}