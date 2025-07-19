import { useEffect } from "react";
import { useApi } from "../lib/api";
import { useNavigate } from "react-router-dom";

export default function DropboxLink() {
  const api = useApi();
  const navigate = useNavigate();

  useEffect(() => {
    const code = new URLSearchParams(window.location.search).get("code");
    // Send code to backend
    if (code) {
      api.linkDropbox(code).then((_success) => {
        navigate("/settings");
      });
    } else {
      navigate("/settings");
    }
  }, []);

  return null;
}
