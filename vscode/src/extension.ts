import * as vscode from "vscode";
import { ensureBinary } from "./downloader";
import { SffFormattingProvider } from "./formatter";

export const activate = async (context: vscode.ExtensionContext) => {
  const outputChannel = vscode.window.createOutputChannel("sff");
  const binaryPath = await ensureBinary(context, outputChannel);
  outputChannel.appendLine(`Binary Path: ${binaryPath}`);
  const provider = new SffFormattingProvider(binaryPath, outputChannel);

  context.subscriptions.push(
    vscode.languages.registerDocumentFormattingEditProvider("python", provider),
    vscode.languages.registerDocumentRangeFormattingEditProvider(
      "python",
      provider,
    ),
  );
};

export const deactivate = () => {};
