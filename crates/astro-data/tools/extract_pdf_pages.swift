// Prints the text layer of a PDF as blocks headed `=== tr. N ===`, the format of a source's
// `packet` (section 3.3 of docs/INTERPRETATION_DB_SPEC.md).
//
// Usage: swift extract_pdf_pages.swift <file.pdf> <printed number of its first page>
//
// Tân Biên comes as six PDFs, starting at printed pages 1, 31, 81, 132, 181 and 225:
//   printf '%s\n' 'dv01 1' 'dv02 31' 'dv03 81' 'dv04 132' 'dv05 181' 'dv06 225' |
//     while read -r part page; do
//       swift crates/astro-data/tools/extract_pdf_pages.swift ".ref/sources/tan_bien/$part.pdf" "$page"
//     done > .ref/sources/tan_bien/pages.txt
import Foundation
import PDFKit

let arguments = CommandLine.arguments
guard arguments.count == 3, let first = Int(arguments[2]) else {
    FileHandle.standardError.write(Data("usage: extract_pdf_pages <file.pdf> <first page>\n".utf8))
    exit(2)
}
guard let document = PDFDocument(url: URL(fileURLWithPath: arguments[1])) else {
    FileHandle.standardError.write(Data("cannot open \(arguments[1])\n".utf8))
    exit(1)
}
for index in 0..<document.pageCount {
    print("=== tr. \(first + index) ===")
    print(document.page(at: index)?.string ?? "")
}
