param([string]$Name = '*')

# The small SVG subset keeps these two precision icons reproducible without extra packages.
Add-Type -AssemblyName System.Drawing
Add-Type -ReferencedAssemblies System.Drawing, System.Xml -TypeDefinition @'
using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Globalization;
using System.Text.RegularExpressions;
using System.Xml;

public static class MoonveilVectorRenderer {
    static float Number(string value) { return float.Parse(value, CultureInfo.InvariantCulture); }

    static GraphicsPath Path(string data) {
        var tokens = Regex.Matches(data, @"[MLCZ]|-?\d+(?:\.\d+)?");
        var path = new GraphicsPath();
        var current = PointF.Empty;
        int i = 0;
        while (i < tokens.Count) {
            string command = tokens[i++].Value;
            if (command == "Z") { path.CloseFigure(); continue; }
            var end = new PointF(Number(tokens[i++].Value), Number(tokens[i++].Value));
            if (command == "M") { path.StartFigure(); }
            else if (command == "L") { path.AddLine(current, end); }
            else if (command == "C") {
                var control = new PointF(Number(tokens[i++].Value), Number(tokens[i++].Value));
                var target = new PointF(Number(tokens[i++].Value), Number(tokens[i++].Value));
                path.AddBezier(current, end, control, target);
                end = target;
            } else { throw new InvalidOperationException("Unsupported SVG command: " + command); }
            current = end;
        }
        return path;
    }

    public static void Render(string source, string dark, string light) {
        var svg = new XmlDocument();
        svg.Load(source);
        using (var large = new Bitmap(512, 512)) {
            using (var graphics = Graphics.FromImage(large)) {
                graphics.SmoothingMode = SmoothingMode.AntiAlias;
                graphics.ScaleTransform(4, 4);
                foreach (XmlElement element in svg.DocumentElement.ChildNodes) {
                    if (element.Name != "path") { throw new InvalidOperationException("Unsupported SVG element"); }
                    using (var path = Path(element.GetAttribute("d"))) {
                        if (element.GetAttribute("fill") == "white") { graphics.FillPath(Brushes.White, path); }
                        if (element.GetAttribute("stroke") == "white") {
                            using (var pen = new Pen(Color.White, Number(element.GetAttribute("stroke-width")))) {
                                pen.LineJoin = LineJoin.Round;
                                if (element.GetAttribute("stroke-linecap") == "round") { pen.StartCap = pen.EndCap = LineCap.Round; }
                                graphics.DrawPath(pen, path);
                            }
                        }
                    }
                }
            }
            using (var white = new Bitmap(128, 128))
            using (var black = new Bitmap(128, 128)) {
                using (var graphics = Graphics.FromImage(white)) {
                    graphics.InterpolationMode = InterpolationMode.HighQualityBicubic;
                    graphics.PixelOffsetMode = PixelOffsetMode.HighQuality;
                    graphics.DrawImage(large, new Rectangle(0, 0, 128, 128));
                }
                for (int y = 0; y < 128; y++) {
                    for (int x = 0; x < 128; x++) {
                        int alpha = x >= 12 && x < 116 && y >= 12 && y < 116 ? white.GetPixel(x, y).A : 0;
                        white.SetPixel(x, y, alpha == 0 ? Color.FromArgb(0, 0, 0, 0) : Color.FromArgb(alpha, 255, 255, 255));
                        black.SetPixel(x, y, Color.FromArgb(alpha, 0, 0, 0));
                    }
                }
                white.Save(dark, System.Drawing.Imaging.ImageFormat.Png);
                black.Save(light, System.Drawing.Imaging.ImageFormat.Png);
            }
        }
    }
}
'@

$iconRoot = Join-Path $PSScriptRoot '../assets/themes/moonveil/icons'
Get-ChildItem -LiteralPath (Join-Path $iconRoot 'sources') -Filter "$Name.svg" | ForEach-Object {
    [MoonveilVectorRenderer]::Render($_.FullName,
        (Join-Path $iconRoot "dark/$($_.BaseName).png"),
        (Join-Path $iconRoot "light/$($_.BaseName).png"))
    Write-Output "Rendered $($_.BaseName)"
}
