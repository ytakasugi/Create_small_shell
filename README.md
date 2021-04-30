# [Build Your Own Shell using Rust](https://www.joshmcguigan.com/blog/build-your-own-shell-rust/)

これは、[build-your-own-x](https://github.com/danistefanovic/build-your-own-x)リストの精神に基づいた、Rustを使って自分のシェルを作るチュートリアルです。シェルを作ることは、シェル、ターミナルエミュレータ、OSがどのように連携するかを理解するのに最適な方法です。

---

### What is a shell?

シェルとは、コンピュータを制御するためのプログラムです。シェルは主に、他のアプリケーションを簡単に起動できるようにすることで、コンピュータをコントロールします。しかし、シェルはそれだけではインタラクティブなアプリケーションとは言えません。

ほとんどのユーザーは、ターミナルエミュレーターを使ってシェルと対話します。ユーザーのgeirha氏によるターミナル・エミュレータの簡潔な説明は以下の通りです。

> ターミナルエミュレータ（単にターミナルと呼ばれることが多い）は、「単なるウィンドウ」です。これはテキストベースのプログラムを実行するもので、デフォルトではログインシェル（`Ubuntu`では`bash`）になっています。あなたがウィンドウに文字を入力すると、ターミナルはその文字をシェル（または他のプログラム）の標準入力に送るだけでなく、ウィンドウに描画します。シェルが`stdout`と`stderr`に出力する文字は、ターミナルに送られ、ターミナルはこれらの文字をウィンドウに描画します。

このチュートリアルでは、独自のシェルを作成し、通常のターミナルエミュレータ内で実行します（通常、`cargo run`する場所であればどこでも構いません）。

---

### A starting Point

最もシンプルなシェルは、わずか数行のRustコードしか必要としません。ここでは、ユーザーの入力を保持するための新しい文字列を作成しています。`stdin().read_line`関数は、ユーザがEnterキーを押すまでブロックし、ユーザの入力内容全体（Enterキーを押したときの改行を含む）を文字列に書き出します。`input.trim()`で改行文字を取り除いた後、コマンドの実行を試みます。

```rust
fn main(){
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    // read_line leaves a trailing newline, which trim removes
    let command = input.trim(); 

    Command::new(command)
        .spawn()
        .unwrap();
}
```

これを実行すると、ターミナルに入力待ちの点滅するカーソルが表示されるはずです。`ls`と入力してEnterキーを押すと、`ls`コマンドがカレントディレクトリの内容を表示し、シェルが終了するのがわかります。

注意：[Rust Playground](https://play.rust-lang.org/)は、stdinや長時間のプロセスをサポートしていないため、これらの例はRust Playgroundでは実行できません。

---

### Accept Multiple Commands

ユーザーが1つのコマンドを入力しただけでシェルが終了するようなことは避けたいものです。複数のコマンドをサポートするには、上のコードをループで囲み、各子プロセスに`wait`の呼び出しを追加することで、現在のプロセスが終了する前にユーザに追加の入力を促さないようにすることがほとんどです。また、ユーザが自分の入力と自分が起動したプロセスの出力とを区別しやすくするために、`>`の文字を表示する行をいくつか追加しました。

```rust
fn main(){
    loop {
        // use the `>` character as the prompt
        // need to explicitly flush this to ensure it prints before read_line
        print!("> ");
        stdout().flush();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let command = input.trim();

        let mut child = Command::new(command)
            .spawn()
            .unwrap();

        // don't accept another command until this one completes
        child.wait(); 
    }
}
```

このコードを実行すると、1つ目のコマンドを実行した後、プロンプトが戻ってきて2つ目のコマンドを入力できることがわかります。例えば、lsやpwdといったコマンドで試してみてください。

---

### Handling Args

上のシェルで`ls -a`というコマンドを実行しようとすると、クラッシュしてしまいます。引数を認識していないので、`ls -a`というコマンドを実行しようとしますが、適切な動作は`ls`というコマンドを引数`-a`で実行することです。

これは以下のように、ユーザーの入力をホワイトスペース文字で分割し、最初のホワイトスペースより前のものはコマンドの名前として扱い（例：`ls`）、最初のホワイトスペースより後のものはそのコマンドに引数として渡される（例：`-a`）ことで修正されます。

```rust
fn main(){
    loop {
        print!("> ");
        stdout().flush();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        // everything after the first whitespace character 
        //     is interpreted as args to the command
        let mut parts = input.trim().split_whitespace();
        let command = parts.next().unwrap();
        let args = parts;

        let mut child = Command::new(command)
            .args(args)
            .spawn()
            .unwrap();

        child.wait();
    }
}
```

---

### Shell Built-ins

シェルが他のプロセスに単純にディスパッチできないコマンドがあることがわかりました。これは、シェルの内部に影響を与えるもので、シェル自身が実装しなければならないものです。

最も一般的な例は`cd`コマンドでしょう。`cd`がシェルのビルトインでなければならない理由については、[こちらのリンク](https://unix.stackexchange.com/a/38809)をご覧ください。シェルのビルトインに加えて、実際には`cd`というプログラムがあります。この二重構造の理由は[こちら](https://unix.stackexchange.com/a/38819)で説明しています。

以下では，シェルに`cd`をビルトインするためのサポートを追加します．

```rust
fn main(){
    loop {
        print!("> ");
        stdout().flush();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let mut parts = input.trim().split_whitespace();
        let command = parts.next().unwrap();
        let args = parts;

        match command {
            "cd" => {
                // default to '/' as new directory if one was not provided
                let new_dir = args.peekable().peek().map_or("/", |x| *x);
                let root = Path::new(new_dir);
                if let Err(e) = env::set_current_dir(&root) {
                    eprintln!("{}", e);
                }
            },
            command => {
                let mut child = Command::new(command)
                    .args(args)
                    .spawn()
                    .unwrap();

                child.wait();
            }
        }
    }
}
```

---

### Error Handling

上記のシェルでは、存在しないコマンドを入力するとクラッシュすることにお気づきでしょうか。以下のバージョンでは、ユーザーにエラーを表示してから別のコマンドを入力できるようにすることで、この問題を優雅に処理しています。

間違ったコマンドを入力することは、シェルを終了させる簡単な方法として機能していたので、もうひとつのシェルの組み込みである`exit`コマンドも実装しました。

```rust
fn main(){
    loop {
        print!("> ");
        stdout().flush();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let mut parts = input.trim().split_whitespace();
        let command = parts.next().unwrap();
        let args = parts;

        match command {
            "cd" => {
                let new_dir = args.peekable().peek().map_or("/", |x| *x);
                let root = Path::new(new_dir);
                if let Err(e) = env::set_current_dir(&root) {
                    eprintln!("{}", e);
                }
            },
            "exit" => return,
            command => {
                let child = Command::new(command)
                    .args(args)
                    .spawn();

                // gracefully handle malformed user input
                match child {
                    Ok(mut child) => { child.wait(); },
                    Err(e) => eprintln!("{}", e),
                };
            }
        }
    }
}
```

---

### Pipes

パイプを含まないシェルで生産性を上げるのは難しいでしょう。この機能に慣れていない方のために説明すると、`｜`文字を使って、最初のコマンドの出力を2番目のコマンドの入力にリダイレクトするようシェルに指示します。例えば、`ls | grep Cargo`というコマンドを実行すると、次のような一連のアクションが発生します。

1. `ls`は、カレントディレクトリ内のすべてのファイルを一覧表示します。

2. シェルは上記のファイルリストを`grep`にパイプします。

3. `grep`は、リストをフィルタリングして、文字列`Cargo`を含むファイルのみを出力します。

このシェルの最終版では、パイプの基本的なサポートが含まれています。パイプやIOリダイレクションでできる他の多くのことについては、[こちら](https://robots.thoughtbot.com/input-output-redirection-in-the-shell)の記事をご覧ください。

```rust
fn main(){
    loop {
        print!("> ");
        stdout().flush();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        // must be peekable so we know when we are on the last command
        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next()  {

            let mut parts = command.trim().split_whitespace();
            let command = parts.next().unwrap();
            let args = parts;

            match command {
                "cd" => {
                    let new_dir = args.peekable().peek()
                        .map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&root) {
                        eprintln!("{}", e);
                    }

                    previous_command = None;
                },
                "exit" => return,
                command => {
                    let stdin = previous_command
                        .map_or(
                            Stdio::inherit(),
                            |output: Child| Stdio::from(output.stdout.unwrap())
                        );

                    let stdout = if commands.peek().is_some() {
                        // there is another command piped behind this one
                        // prepare to send output to the next command
                        Stdio::piped()
                    } else {
                        // there are no more commands piped behind this one
                        // send output to shell stdout
                        Stdio::inherit()
                    };

                    let output = Command::new(command)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    match output {
                        Ok(output) => { previous_command = Some(output); },
                        Err(e) => {
                            previous_command = None;
                            eprintln!("{}", e);
                        },
                    };
                }
            }
        }

        if let Some(mut final_command) = previous_command {
            // block until the final command has finished
            final_command.wait();
        }

    }
}
```

---

### Conclusion

100行にも満たないRustで、日常的な作業に使えるシェルを作成しましたが、実際のシェルにはもっと多くの機能があります。GNUのウェブサイトには、bashシェルのオンラインマニュアルがあり、その中には[シェルの機能一覧](https://www.gnu.org/software/bash/manual/html_node/Basic-Shell-Features.html#Basic-Shell-Features)がありますので、より高度な機能を調べ始めるには最適です。

これは私にとって学習プロジェクトであり、シンプルさと堅牢性がトレードオフになる場合、私はほとんどの場合、シンプルさを選択していることに留意してください。

このシェルは[GitHub](https://github.com/JoshMcguigan/bubble-shell)で公開されています。この記事を書いている時点での最新のコミットは[a47640](https://github.com/JoshMcguigan/bubble-shell/tree/a6b81d837e4f5e68cf0b72a4d55e95fb08a47640)です。Rustシェルの学習プロジェクトとしては、他にも[Rush](https://github.com/psinghal20/rush)がありますが、こちらも興味深いです。
