import * as vscode from "vscode";
import { execFile } from "child_process";
import { promisify } from "util";

const execFileAsync = promisify(execFile);

export class SffFormattingProvider
  implements
    vscode.DocumentFormattingEditProvider,
    vscode.DocumentRangeFormattingEditProvider
{
  constructor(
    private binaryPath: string,
    private output: vscode.OutputChannel,
  ) {}

  async provideDocumentFormattingEdits(
    document: vscode.TextDocument,
  ): Promise<vscode.TextEdit[]> {
    return this.format(document);
  }

  async provideDocumentRangeFormattingEdits(
    document: vscode.TextDocument,
    range: vscode.Range,
  ): Promise<vscode.TextEdit[]> {
    return this.format(document, range);
  }

  private async format(
    document: vscode.TextDocument,
    range?: vscode.Range,
  ): Promise<vscode.TextEdit[]> {
    const config = vscode.workspace.getConfiguration("sff");

    const args: string[] = [];
    const lineLength = config.get<number | null>("lineLength");
    const logLevel = config.get<string | null>("logLevel");

    if (lineLength !== undefined && lineLength !== null) {
      args.push("--line-length", String(lineLength));
    }
    const text = range ? document.getText(range) : document.getText();

    this.output.appendLine(
      `Formatting text of length ${text.length} with args: ${args.join(" ")}`,
    );

    try {
      const proc = execFileAsync(this.binaryPath, args, {
        env: { ...process.env, ...(logLevel ? { RUST_LOG: logLevel } : {}) },
      });

      proc.child.stdin!.write(text);
      proc.child.stdin!.end();

      const { stdout, stderr } = await proc;

      if (stderr) {
        this.output.appendLine(stderr);
      }

      const editRange =
        range ??
        new vscode.Range(
          document.positionAt(0),
          document.positionAt(document.getText().length),
        );
      return [vscode.TextEdit.replace(editRange, stdout)];
    } catch (err: any) {
      const message = err.stderr ?? err.message;
      this.output.appendLine(`Error: ${message}`);
      vscode.window.showErrorMessage(`sff-formatter: ${message}`);
      return [];
    }
  }
}
