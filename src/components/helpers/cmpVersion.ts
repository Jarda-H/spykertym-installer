export function cmpVersions(v1: string, v2: string): number {
  var v1parts = v1.split(".");
  var v2parts = v2.split(".");
  var maxLen = Math.max(v1parts.length, v2parts.length);
  for (var i = 0; i < maxLen; i++) {
    var v1part = parseInt(v1parts[i], 10);
    var v2part = parseInt(v2parts[i], 10);
    if (v1part < v2part) {
      return 0;
    }
    if (v1part > v2part) {
      return 1;
    }
  }
  return 0;
}
