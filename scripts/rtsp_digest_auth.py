import hashlib
import socket
import re

def calculate_digest_response(username, password, realm, nonce, method, uri):
    """计算Digest认证的response值"""
    # 计算HA1: MD5(username:realm:password)
    ha1_str = f"{username}:{realm}:{password}"
    ha1 = hashlib.md5(ha1_str.encode('utf-8')).hexdigest()
    
    # 计算HA2: MD5(method:uri)
    ha2_str = f"{method}:{uri}"
    ha2 = hashlib.md5(ha2_str.encode('utf-8')).hexdigest()
    
    # 计算response: MD5(HA1:nonce:HA2)
    response_str = f"{ha1}:{nonce}:{ha2}"
    response = hashlib.md5(response_str.encode('utf-8')).hexdigest()
    
    return response

def rtsp_describe_with_digest(rtsp_url, username, password):
    # 解析RTSP URL（提取主机、端口、路径）
    match = re.match(r'rtsp://([^:]+):?(\d+)?(/.*)', rtsp_url)
    if not match:
        raise ValueError("Invalid RTSP URL")
    host, port, uri = match.groups()
    port = int(port) if port else 554  # RTSP默认端口554
    
    # 1. 发送初始DESCRIBE请求（无认证）
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect((host, port))
    cseq = 1  # RTSP的CSeq序列号（递增）
    initial_request = (
        f"DESCRIBE {uri} RTSP/1.0\r\n"
        f"CSeq: {cseq}\r\n"
        f"User-Agent: Python-RTSP-Client\r\n"
        f"Accept: application/sdp\r\n\r\n"
    )
    sock.send(initial_request.encode('utf-8'))
    
    # 2. 接收服务器响应，提取401和认证参数
    response = sock.recv(4096).decode('utf-8')
    if '401 Unauthorized' not in response:
        raise Exception("Server did not request authentication")
    
    # 解析WWW-Authenticate头（提取realm和nonce）
    auth_header = re.search(r'WWW-Authenticate: Digest (.*?)\r\n', response, re.DOTALL).group(1)
    realm = re.search(r'realm="(.*?)"', auth_header).group(1)
    nonce = re.search(r'nonce="(.*?)"', auth_header).group(1)
    
    # 3. 计算Digest响应值
    method = "DESCRIBE"
    response_val = calculate_digest_response(username, password, realm, nonce, method, uri)
    
    # 4. 构建带认证的Authorization头，重新发送请求
    cseq += 1  # 递增CSeq
    auth_request = (
        f"DESCRIBE {uri} RTSP/1.0\r\n"
        f"CSeq: {cseq}\r\n"
        f"User-Agent: Python-RTSP-Client\r\n"
        f"Accept: application/sdp\r\n"
        f"Authorization: Digest username=\"{username}\", realm=\"{realm}\", "
        f"nonce=\"{nonce}\", uri=\"{uri}\", response=\"{response_val}\"\r\n\r\n"
    )
    print(auth_header)
    sock.send(auth_request.encode('utf-8'))
    
    # 5. 接收最终响应（如200 OK及SDP内容）
    final_response = sock.recv(4096).decode('utf-8')
    sock.close()
    return final_response

# 使用示例
if __name__ == "__main__":
    rtsp_url = "rtsp://60.243.26.171:555/"  # 替换为实际RTSP地址
    username = "admin"  # 替换为实际用户名
    password = "123456"  # 替换为实际密码
    
    try:
        result = rtsp_describe_with_digest(rtsp_url, username, password)
        print("RTSP DESCRIBE响应:\n", result)
    except Exception as e:
        print("错误:", e)
