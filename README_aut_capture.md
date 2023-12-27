static function OnBeforeResponse(oSession: Session) {
        if (m_Hide304s && oSession.responseCode == 304) {
            oSession["ui-hide"] = "true";
        }
		// if (oSession.fullUrl.Contains("baidu.com")){
        if (oSession.fullUrl.Contains("1000mz.com")&& 1){
            oSession.utilDecodeResponse();//消除保存的请求可能存在乱码的情况
            var jsonString = oSession.GetResponseBodyAsString();
            var responseJSON = Fiddler.WebFormats.JSON.JsonDecode(jsonString);
            if((responseJSON.JSONObject=='System.Collections.ArrayList' || responseJSON.JSONObject=='System.Collections.Hashtable')&&jsonString!='[]'&&jsonString!='{}'){
                // 判断是否是json数据 然后保存

                var str='{}';//构造自己的JSON http请求的信息及返回的结果
                var data = Fiddler.WebFormats.JSON.JsonDecode(str);
                data.JSONObject["request_method"] = oSession.RequestMethod;
                var requestString = oSession.GetRequestBodyAsString();

                data.JSONObject["request_body"]= requestString;
                data.JSONObject["response_data"] = responseJSON.JSONObject;
                data.JSONObject["url"] = oSession.fullUrl;
                data.JSONObject["response_code"] = oSession.responseCode;
                jsonString = Fiddler.WebFormats.JSON.JsonEncode(data.JSONObject)

                // 保存文件到本地
                var fso;
                var file;
                fso = new ActiveXObject("Scripting.FileSystemObject");
                file = fso.OpenTextFile("H:\\RustProject\\auto_tools\\j2s\\json\\cms\\Sessions.dat",8 ,true, true);
                file.writeLine(jsonString);
                file.writeLine("\n");
                file.close();


            }
        }-


// 自动试用FIDDLE 保存网站中的json数据并保存到本地文件中
fiddler Menu -> Rules-> Custom Rules
<!-- 
脚本中的主要方法说明：
static function OnBoot fiddler 启动时调用
static function OnShutdown fiddler关闭时调用
static function OnAttach fiddler注册成系统代理时调用
static function OnDetach fiddler 取消注册系统代理时调用
static function Main 在每次fiddler启动时和编译CustomRules.js 脚本时调用。
static function OnBeforeRequest(oSession: Session) 在这个方法中修改Request的内容
static function OnBeforeResponse(oSession: Session) 在这个方法中修改Response的内容（对应爬虫来说该方法最常用）                

// 数据通过post请求发送自己的后台服务保存 
                FiddlerObject.log('2222222222222222'+jsonString); 
                // methods
                var method = "POST";
                var myUrl = 'http://localhost:8000/fiddler'
                var url = myUrl+'?data='+Utilities.UrlEncode(jsonString);
                var protocol = "HTTP/1.1";
                var raw="";
                var selected: Session = oSession;
                raw += method + " " + url + " " + protocol + "\r\n";
  
                raw +="Host:localhost:8000\r\n";
                raw +="Connection: keep-alive\r\n";
                raw +="Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8\r\n";
                raw +="User-Agent: Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/38.0.2125.122 Safari/537.36\r\n";
                raw +="Accept-Encoding: gzip,deflate,sdch\r\n";
                raw +="Accept-Language: zh-CN,zh;q=0.8,en-US;q=0.6,en;q=0.4\r\n";
                raw +="Content-Type: application/json\r\n";
 
                var body= "jsondata=''";
                raw += "\r\n" + body;
                FiddlerObject.utilIssueRequest(raw); -->