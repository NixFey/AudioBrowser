using System.Runtime.InteropServices;
using Mono.Unix.Native;

namespace AudioBrowser.Services;

public class XAttrsService
{
    public static bool SetBool(FileInfo file, string attrName, bool value)
    {
        var result = Syscall.setxattr(file.FullName, "user." + attrName, [value ? (byte)1 : (byte)0], XattrFlags.XATTR_AUTO);

        if (result == 0) return true;

        var errno = Stdlib.GetLastError();
        
        throw new ApplicationException($"Failed to set xattr on file: {result} {errno}");
    }
    
    public static bool? GetBool(FileInfo file, string attrName)
    {
        var result = Syscall.getxattr(file.FullName, "user." + attrName, out var data);

        if (result == -1)
        {
            var errno = Stdlib.GetLastError();

            if (errno == Errno.ENODATA) return null;
            
            throw new ApplicationException($"Failed to get xattr on file: {result} {errno}");
        }

        if (data.Length != 1)
            throw new ApplicationException($"Expected a boolean value but got data with length {data.Length}");

        return data[0] switch
        {
            0 => false,
            1 => true,
            _ => throw new ApplicationException($"Got unexpected value from xattr {data[0]}")
        };
    }
}

internal static class Interop
{
    [DllImport("libc", SetLastError = true)]
    internal static extern unsafe int setxattr(
        [MarshalAs(UnmanagedType.LPWStr)] string path,
        [MarshalAs(UnmanagedType.LPWStr)] string name,
        void* value,
        ulong size,
        int flags);
}